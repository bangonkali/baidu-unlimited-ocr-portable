impl AppState {
    async fn execute_generate_embedding(
        &self,
        request: &GenerateEmbeddingRequest,
        task: &PipelineTaskRow,
    ) -> Result<GenerateEmbeddingResponse> {
        let setup = self.prepare_embedding_execution(request, task).await?;
        let inserted = self
            .execute_embedding_pages(EmbeddingPagesInput {
                request,
                task,
                task_scope: &setup.task_scope,
                embedding_run_id: &setup.embedding_run_id,
                dimension: setup.dimension,
                segments: &setup.segments,
                started_at: &setup.started_at,
                profile: &setup.profile,
            })
            .await?;
        let finished_task = self
            .finish_generate_embedding_task(FinishEmbeddingTaskInput {
                request,
                task,
                task_scope: &setup.task_scope,
                embedding_run_id: &setup.embedding_run_id,
                dimension: setup.dimension,
                started_at: setup.started_at,
                segments_total: usize_to_u32_saturating(setup.segments.len()),
                inserted,
            })
            .await?;
        Ok(GenerateEmbeddingResponse {
            task: pipeline_task_record(finished_task),
            embedding_run_id: setup.embedding_run_id,
            source_run_id: request.source_run_id.clone(),
            model_id: request.model_id.clone(),
            dimension: setup.dimension,
            segments_embedded: inserted,
            status: "completed".to_string(),
        })
    }
}
