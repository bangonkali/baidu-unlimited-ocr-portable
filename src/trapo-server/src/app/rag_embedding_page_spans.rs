impl AppState {
    async fn start_embedding_page_span(&self, context: &EmbeddingPageContext<'_>) -> Result<()> {
        self.upsert_rag_diagnostic_span(
            context.page_scope,
            embedding_page_span(context),
            "running",
            None,
            json!({"segment_count": context.page.segments.len(), "model_id": context.request.model_id}),
        )
        .await
    }

    async fn generate_page_embeddings(
        &self,
        context: &EmbeddingPageContext<'_>,
    ) -> Result<Vec<Vec<f32>>> {
        let texts = context
            .page
            .segments
            .iter()
            .map(|segment| segment.text.clone())
            .collect::<Vec<_>>();
        match generate_embeddings(context.profile.clone(), EmbeddingPurpose::Document, texts).await {
            Ok(embeddings) => Ok(embeddings),
            Err(error) => {
                let message = error.to_string();
                self.fail_embedding_page(context, &message).await?;
                Err(error)
            }
        }
    }

    async fn insert_page_embedding_vectors(
        &self,
        context: &EmbeddingPageContext<'_>,
        vector_rows: &[RagEmbeddingVectorRow],
    ) -> Result<u32> {
        match self
            .inner
            .repository
            .insert_rag_embedding_vectors(context.dimension, vector_rows)
            .await
        {
            Ok(inserted) => Ok(inserted),
            Err(error) => {
                let message = error.to_string();
                self.fail_embedding_page(context, &message).await?;
                Err(error)
            }
        }
    }

    async fn finish_embedding_page_span(
        &self,
        context: &EmbeddingPageContext<'_>,
        inserted_page: u32,
    ) -> Result<()> {
        self.upsert_rag_diagnostic_span(
            context.page_scope,
            embedding_page_span(context),
            "ok",
            None,
            json!({"segments_embedded": inserted_page, "model_id": context.request.model_id}),
        )
        .await
    }

    async fn fail_embedding_page(
        &self,
        context: &EmbeddingPageContext<'_>,
        message: &str,
    ) -> Result<()> {
        self.upsert_rag_diagnostic_span(
            context.page_scope,
            embedding_page_span(context),
            "failed",
            Some(message),
            json!({"model_id": context.request.model_id}),
        )
        .await?;
        self.upsert_rag_diagnostic_span(
            context.task_scope,
            RagSpanInput::task(context.task, "generate_embedding"),
            "failed",
            Some(message),
            json!({"model_id": context.request.model_id}),
        )
        .await
    }

    async fn finish_generate_embedding_task(
        &self,
        input: FinishEmbeddingTaskInput<'_>,
    ) -> Result<PipelineTaskRow> {
        self.inner
            .repository
            .upsert_rag_embedding_run(&completed_embedding_run(CompletedEmbeddingRunInput {
                embedding_run_id: input.embedding_run_id,
                request: input.request,
                task: input.task,
                started_at: input.started_at,
                dimension: input.dimension,
                segments_total: input.segments_total,
                inserted: input.inserted,
            }))
            .await?;
        let finished_task = self
            .inner
            .repository
            .finish_pipeline_task(
                &input.task.task_id,
                "completed",
                &json!({"segments_embedded": input.inserted, "model_id": input.request.model_id}),
                None,
            )
            .await?;
        self.upsert_rag_diagnostic_span(
            input.task_scope,
            RagSpanInput::task(input.task, "generate_embedding"),
            "ok",
            None,
            json!({"segments_embedded": input.inserted, "model_id": input.request.model_id, "embedding_run_id": input.embedding_run_id}),
        )
        .await?;
        Ok(finished_task)
    }
}

struct FinishEmbeddingTaskInput<'a> {
    request: &'a GenerateEmbeddingRequest,
    task: &'a PipelineTaskRow,
    task_scope: &'a DiagnosticSpanScope,
    embedding_run_id: &'a str,
    dimension: u32,
    started_at: String,
    segments_total: u32,
    inserted: u32,
}

fn embedding_page_span<'a>(context: &'a EmbeddingPageContext<'a>) -> RagSpanInput<'a> {
    RagSpanInput::page(RagPageSpanInput {
        task: context.task,
        parent_span_id: &context.task_scope.span_id,
        file_hash: &context.page.file_hash,
        page_no: context.page.page_no,
        name: "Generate page embeddings",
        pipeline_step: "generate_embedding",
        span_kind: "embedding_page",
    })
}
