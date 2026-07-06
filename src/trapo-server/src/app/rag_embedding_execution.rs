struct PreparedEmbeddingExecution {
    dimension: u32,
    embedding_run_id: String,
    profile: LlamaEmbeddingProfile,
    segments: Vec<RagTextSegmentRow>,
    started_at: String,
    task_scope: DiagnosticSpanScope,
}

struct EmbeddingPagesInput<'a> {
    request: &'a GenerateEmbeddingRequest,
    task: &'a PipelineTaskRow,
    task_scope: &'a DiagnosticSpanScope,
    embedding_run_id: &'a str,
    dimension: u32,
    segments: &'a [RagTextSegmentRow],
    started_at: &'a str,
    profile: &'a LlamaEmbeddingProfile,
}

struct EmbeddingPageContext<'a> {
    request: &'a GenerateEmbeddingRequest,
    task: &'a PipelineTaskRow,
    task_scope: &'a DiagnosticSpanScope,
    page_scope: &'a DiagnosticSpanScope,
    embedding_run_id: &'a str,
    dimension: u32,
    page: &'a PageSegmentGroup<'a>,
    profile: &'a LlamaEmbeddingProfile,
}

impl AppState {
    async fn prepare_embedding_execution(
        &self,
        request: &GenerateEmbeddingRequest,
        task: &PipelineTaskRow,
    ) -> Result<PreparedEmbeddingExecution> {
        let model = self.embedding_model(&request.model_id).await?;
        let dimension = request.dimension.unwrap_or(model.dimension);
        let task_scope = DiagnosticSpanScope::start();
        self.upsert_rag_diagnostic_span(
            &task_scope,
            RagSpanInput::task(task, "generate_embedding"),
            "running",
            None,
            json!({"model_id": request.model_id, "dimension": dimension}),
        )
        .await?;
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
        let profile = self.embedding_profile_for_model(&model, dimension).await?;
        Ok(PreparedEmbeddingExecution {
            dimension,
            embedding_run_id,
            profile,
            segments,
            started_at,
            task_scope,
        })
    }

    async fn embedding_profile_for_model(
        &self,
        model: &RagEmbeddingModelRow,
        dimension: u32,
    ) -> Result<LlamaEmbeddingProfile> {
        let runtime_id = self.selected_embedding_runtime_id().await;
        let mut profile = profile_from_model_row(
            &self.inner.config.app_root,
            &self.inner.config.model_dir,
            model,
            Some(&runtime_id),
        )?;
        profile.dimension = dimension;
        Ok(profile)
    }

    async fn execute_embedding_pages(&self, input: EmbeddingPagesInput<'_>) -> Result<u32> {
        let mut inserted = 0_u32;
        for page in page_segment_groups(input.segments) {
            let page_scope = DiagnosticSpanScope::start();
            let context = EmbeddingPageContext {
                request: input.request,
                task: input.task,
                task_scope: input.task_scope,
                page_scope: &page_scope,
                embedding_run_id: input.embedding_run_id,
                dimension: input.dimension,
                page: &page,
                profile: input.profile,
            };
            let inserted_page = self.execute_embedding_page(&context).await?;
            inserted = inserted.saturating_add(inserted_page);
            self.record_embedding_progress(&input, inserted).await?;
        }
        Ok(inserted)
    }

    async fn execute_embedding_page(&self, context: &EmbeddingPageContext<'_>) -> Result<u32> {
        self.start_embedding_page_span(context).await?;
        let embeddings = self.generate_page_embeddings(context).await?;
        let vector_rows = embedding_vector_rows(context, embeddings);
        let inserted_page = self.insert_page_embedding_vectors(context, &vector_rows).await?;
        self.finish_embedding_page_span(context, inserted_page).await?;
        Ok(inserted_page)
    }

    async fn record_embedding_progress(
        &self,
        input: &EmbeddingPagesInput<'_>,
        inserted: u32,
    ) -> Result<()> {
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
                segments_total: usize_to_u32_saturating(input.segments.len()),
                segments_embedded: inserted,
                started_at: input.started_at.to_string(),
                finished_at: None,
                error: None,
                params: json!({"model_id": input.request.model_id, "dimension": input.dimension}),
            })
            .await
    }
}

fn embedding_vector_rows(
    context: &EmbeddingPageContext<'_>,
    embeddings: Vec<Vec<f32>>,
) -> Vec<RagEmbeddingVectorRow> {
    context
        .page
        .segments
        .iter()
        .zip(embeddings)
        .map(|(segment, embedding)| RagEmbeddingVectorRow {
            embedding_run_id: context.embedding_run_id.to_string(),
            source_run_id: context.request.source_run_id.clone(),
            segment_id: segment.segment_id.clone(),
            model_id: context.request.model_id.clone(),
            file_hash: segment.file_hash.clone(),
            page_no: segment.page_no,
            embedding,
            metadata: json!({"category": segment.category}),
        })
        .collect()
}
