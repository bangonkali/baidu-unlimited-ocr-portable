impl AppState {
    async fn execute_text_index(
        &self,
        request: &TextIndexRequest,
        task: &PipelineTaskRow,
    ) -> Result<TextIndexResponse> {
        let started_at = Utc::now().to_rfc3339();
        let text_index_run_id = new_persistence_id();
        let segments = self.materialize_rag_text_segments(&request.source_run_id).await?;
        let segments_indexed = self
            .inner
            .repository
            .replace_rag_text_segments(&request.source_run_id, &segments)
            .await?;
        self.inner
            .repository
            .upsert_rag_text_index_run(&RagTextIndexRunRow {
                text_index_run_id: text_index_run_id.clone(),
                task_id: Some(task.task_id.clone()),
                source_run_id: request.source_run_id.clone(),
                status: "running".to_string(),
                segments_indexed,
                started_at: started_at.clone(),
                finished_at: None,
                error: None,
            })
            .await?;
        let fts_segments = self
            .inner
            .repository
            .refresh_rag_fts_index(&text_index_run_id, &request.source_run_id)
            .await?;
        self.inner
            .repository
            .upsert_rag_text_index_run(&RagTextIndexRunRow {
                text_index_run_id: text_index_run_id.clone(),
                task_id: Some(task.task_id.clone()),
                source_run_id: request.source_run_id.clone(),
                status: "completed".to_string(),
                segments_indexed: fts_segments,
                started_at,
                finished_at: Some(Utc::now().to_rfc3339()),
                error: None,
            })
            .await?;
        let finished_task = self
            .inner
            .repository
            .finish_pipeline_task(
                &task.task_id,
                "completed",
                &json!({"segments_indexed": fts_segments}),
                None,
            )
            .await?;
        Ok(TextIndexResponse {
            task: pipeline_task_record(finished_task),
            text_index_run_id,
            source_run_id: request.source_run_id.clone(),
            segments_indexed: fts_segments,
            status: "completed".to_string(),
        })
    }

    async fn execute_generate_embedding(
        &self,
        request: &GenerateEmbeddingRequest,
        task: &PipelineTaskRow,
    ) -> Result<GenerateEmbeddingResponse> {
        let model = self.embedding_model(&request.model_id).await?;
        let dimension = request.dimension.unwrap_or(model.dimension);
        let segments = self.ensure_rag_text_segments(&request.source_run_id).await?;
        let started_at = Utc::now().to_rfc3339();
        let embedding_run_id = new_persistence_id();
        self.start_embedding_run_record(RunningEmbeddingRunInput {
            request,
            task,
            embedding_run_id: &embedding_run_id,
            dimension,
            segments_total: usize_to_u32_saturating(segments.len()),
            started_at: &started_at,
        })
        .await?;
        let mut profile = profile_from_model_row(
            &self.inner.config.app_root,
            &self.inner.config.model_dir,
            &model,
        )?;
        profile.dimension = dimension;
        let texts = segments
            .iter()
            .map(|segment| segment.text.clone())
            .collect::<Vec<_>>();
        let embeddings = generate_embeddings(profile, EmbeddingPurpose::Document, texts).await?;
        let vector_rows = segments
            .iter()
            .zip(embeddings)
            .map(|(segment, embedding)| RagEmbeddingVectorRow {
                embedding_run_id: embedding_run_id.clone(),
                source_run_id: request.source_run_id.clone(),
                segment_id: segment.segment_id.clone(),
                model_id: request.model_id.clone(),
                file_hash: segment.file_hash.clone(),
                page_no: segment.page_no,
                embedding,
                metadata: json!({"category": segment.category}),
            })
            .collect::<Vec<_>>();
        let inserted = self
            .inner
            .repository
            .insert_rag_embedding_vectors(dimension, &vector_rows)
            .await?;
        self.inner
            .repository
            .upsert_rag_embedding_run(&completed_embedding_run(CompletedEmbeddingRunInput {
                embedding_run_id: &embedding_run_id,
                request,
                task,
                started_at,
                dimension,
                segments_total: usize_to_u32_saturating(segments.len()),
                inserted,
            }))
            .await?;
        let finished_task = self
            .inner
            .repository
            .finish_pipeline_task(
                &task.task_id,
                "completed",
                &json!({"segments_embedded": inserted, "model_id": request.model_id}),
                None,
            )
            .await?;
        Ok(GenerateEmbeddingResponse {
            task: pipeline_task_record(finished_task),
            embedding_run_id,
            source_run_id: request.source_run_id.clone(),
            model_id: request.model_id.clone(),
            dimension,
            segments_embedded: inserted,
            status: "completed".to_string(),
        })
    }

    async fn ensure_rag_text_segments(&self, source_run_id: &str) -> Result<Vec<RagTextSegmentRow>> {
        let existing = self
            .inner
            .repository
            .load_rag_text_segments(source_run_id)
            .await?;
        if !existing.is_empty() {
            return Ok(existing);
        }
        let segments = self.materialize_rag_text_segments(source_run_id).await?;
        self.inner
            .repository
            .replace_rag_text_segments(source_run_id, &segments)
            .await?;
        Ok(segments)
    }

    async fn materialize_rag_text_segments(
        &self,
        source_run_id: &str,
    ) -> Result<Vec<RagTextSegmentRow>> {
        let file_hashes = self.file_hashes_for_run(source_run_id).await?;
        let mut segments = Vec::new();
        for file_hash in file_hashes {
            let pages = self
                .inner
                .repository
                .load_document_text_for_run(&file_hash, source_run_id)
                .await?;
            append_page_segments(source_run_id, &file_hash, pages, &mut segments);
        }
        Ok(segments)
    }

    async fn file_hashes_for_run(&self, run_id: &str) -> Result<Vec<String>> {
        let snapshot = self.inner.repository.load_snapshot().await?;
        let hashes = snapshot
            .run_documents
            .into_iter()
            .filter(|item| item.run_id == run_id)
            .map(|item| item.file_hash)
            .collect::<Vec<_>>();
        if hashes.is_empty() {
            return Err(AppError::NotFound(format!("run has no indexed documents: {run_id}")));
        }
        Ok(hashes)
    }

    async fn embedding_model(&self, model_id: &str) -> Result<RagEmbeddingModelRow> {
        self.inner
            .repository
            .list_rag_embedding_models()
            .await?
            .into_iter()
            .find(|model| model.model_id == model_id)
            .ok_or_else(|| AppError::BadRequest(format!("unknown embedding model: {model_id}")))
    }

    async fn start_embedding_run_record(&self, input: RunningEmbeddingRunInput<'_>) -> Result<()> {
        self.inner
            .repository
            .upsert_rag_embedding_run(&RagEmbeddingRunRow {
                embedding_run_id: input.embedding_run_id.to_string(),
                task_id: Some(input.task.task_id.clone()),
                source_run_id: input.request.source_run_id.clone(),
                model_id: input.request.model_id.clone(),
                requested_dimension: input.dimension,
                actual_dimension: input.dimension,
                status: "running".to_string(),
                segments_total: input.segments_total,
                segments_embedded: 0,
                started_at: input.started_at.to_string(),
                finished_at: None,
                error: None,
                params: json!({"model_id": input.request.model_id, "dimension": input.dimension}),
            })
            .await
    }
}

struct RunningEmbeddingRunInput<'a> {
    request: &'a GenerateEmbeddingRequest,
    task: &'a PipelineTaskRow,
    embedding_run_id: &'a str,
    dimension: u32,
    segments_total: u32,
    started_at: &'a str,
}

struct CompletedEmbeddingRunInput<'a> {
    embedding_run_id: &'a str,
    request: &'a GenerateEmbeddingRequest,
    task: &'a PipelineTaskRow,
    started_at: String,
    dimension: u32,
    segments_total: u32,
    inserted: u32,
}

fn completed_embedding_run(input: CompletedEmbeddingRunInput<'_>) -> RagEmbeddingRunRow {
    RagEmbeddingRunRow {
        embedding_run_id: input.embedding_run_id.to_string(),
        task_id: Some(input.task.task_id.clone()),
        source_run_id: input.request.source_run_id.clone(),
        model_id: input.request.model_id.clone(),
        requested_dimension: input.dimension,
        actual_dimension: input.dimension,
        status: "completed".to_string(),
        segments_total: input.segments_total,
        segments_embedded: input.inserted,
        started_at: input.started_at,
        finished_at: Some(Utc::now().to_rfc3339()),
        error: None,
        params: json!({"model_id": input.request.model_id, "dimension": input.dimension}),
    }
}
