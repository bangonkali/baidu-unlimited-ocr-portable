impl AppState {
    async fn ensure_rag_text_segments(&self, source_run_id: &str) -> Result<Vec<RagTextSegmentRow>> {
        let existing = self
            .inner
            .repository
            .load_rag_text_segments(source_run_id)
            .await?;
        if rag_text_segments_are_current(&existing) {
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
        self.materialize_rag_text_segments_with_diagnostics(source_run_id, None)
            .await
    }

    async fn materialize_rag_text_segments_with_diagnostics(
        &self,
        source_run_id: &str,
        diagnostics: Option<RagTaskDiagnosticContext<'_>>,
    ) -> Result<Vec<RagTextSegmentRow>> {
        let file_hashes = self.file_hashes_for_run(source_run_id).await?;
        let mut segments = Vec::new();
        for file_hash in file_hashes {
            let pages = self.rag_pages_for_file(source_run_id, &file_hash).await?;
            for page in pages {
                let span = diagnostics.as_ref().map(|context| {
                    (
                        DiagnosticSpanScope::start(),
                        *context,
                        page.page_no,
                        segments.len(),
                    )
                });
                append_page_segments(source_run_id, &file_hash, vec![page], &mut segments);
                if let Some((scope, context, page_no, start_len)) = span {
                    self.upsert_rag_diagnostic_span(
                        &scope,
                        RagSpanInput::page(RagPageSpanInput {
                            task: context.task,
                            parent_span_id: context.parent_span_id,
                            file_hash: &file_hash,
                            page_no,
                            name: "Segment page text",
                            pipeline_step: context.pipeline_step,
                            span_kind: "text_segment_page",
                        }),
                        "ok",
                        None,
                        json!({"segments_added": segments.len().saturating_sub(start_len)}),
                    )
                    .await?;
                }
            }
        }
        Ok(segments)
    }

    async fn rag_pages_for_file(
        &self,
        source_run_id: &str,
        file_hash: &str,
    ) -> Result<Vec<PageTextRecord>> {
        if let Some(result) = self.preferred_rag_preview_result(source_run_id, file_hash).await? {
            let pages = self
                .inner
                .repository
                .load_document_text_for_run_engine(file_hash, &result.run_engine_id)
                .await?;
            if !pages.is_empty() {
                return Ok(pages);
            }
        }
        self.inner
            .repository
            .load_document_text_for_run(file_hash, source_run_id)
            .await
    }

    async fn preferred_rag_preview_result(
        &self,
        source_run_id: &str,
        file_hash: &str,
    ) -> Result<Option<StoredPreviewResult>> {
        let results = self
            .inner
            .repository
            .preview_results_for_document(source_run_id, file_hash)
            .await?;
        Ok(results
            .iter()
            .find(|result| result.engine_kind == "document_understanding")
            .or_else(|| results.first())
            .cloned())
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
            return Err(AppError::NotFound(format!(
                "run has no indexed documents: {run_id}"
            )));
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
