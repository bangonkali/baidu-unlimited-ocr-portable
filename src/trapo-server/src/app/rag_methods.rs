impl AppState {
    async fn selected_embedding_runtime_id(&self) -> String {
        self.inner.state.lock().await.selected_runtime_id.clone()
    }

    pub(crate) async fn ensure_no_active_pipeline_task(&self) -> Result<()> {
        if let Some(task) = self.inner.repository.active_pipeline_task().await? {
            return Err(AppError::Conflict(format!(
                "another task is already active: {}",
                task.task_kind
            )));
        }
        Ok(())
    }

    pub(crate) async fn start_text_index(
        &self,
        request: TextIndexRequest,
    ) -> Result<TextIndexResponse> {
        self.ensure_not_shutting_down()?;
        self.ensure_no_active_ingest().await?;
        let params = json!({"source_run_id": request.source_run_id});
        let task = self
            .inner
            .repository
            .create_pipeline_task("text_index", Some(&request.source_run_id), &params)
            .await?;
        let task = self
            .inner
            .repository
            .start_pipeline_task(&task.task_id, "local-runner-1")
            .await?;
        let result = self.execute_text_index(&request, &task).await;
        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let _ = self
                    .inner
                    .repository
                    .finish_pipeline_task(
                        &task.task_id,
                        "failed",
                        &json!({"error": error.to_string()}),
                        Some(error.to_string().as_str()),
                    )
                    .await;
                Err(error)
            }
        }
    }

    pub(crate) async fn start_generate_embedding(
        &self,
        request: GenerateEmbeddingRequest,
    ) -> Result<GenerateEmbeddingResponse> {
        self.ensure_not_shutting_down()?;
        self.ensure_no_active_ingest().await?;
        let params = serde_json::to_value(&request)?;
        let task = self
            .inner
            .repository
            .create_pipeline_task("generate_embedding", Some(&request.source_run_id), &params)
            .await?;
        let task = self
            .inner
            .repository
            .start_pipeline_task(&task.task_id, "local-runner-1")
            .await?;
        let result = self.execute_generate_embedding(&request, &task).await;
        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let _ = self
                    .inner
                    .repository
                    .finish_pipeline_task(
                        &task.task_id,
                        "failed",
                        &json!({"error": error.to_string()}),
                        Some(error.to_string().as_str()),
                    )
                    .await;
                Err(error)
            }
        }
    }

    pub(crate) async fn used_embedding_models(&self) -> Result<UsedEmbeddingModelsPayload> {
        let models = self.inner.repository.list_used_rag_embedding_models().await?;
        Ok(UsedEmbeddingModelsPayload {
            models: models
                .into_iter()
                .map(|model| UsedEmbeddingModelRecord {
                    model_id: model.model_id,
                    display_name: model.display_name,
                    dimension: model.dimension,
                    provider: model.provider,
                })
                .collect(),
        })
    }

    pub(crate) async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> Result<HybridSearchResponse> {
        let limit = request.limit.unwrap_or(50).clamp(1, 200);
        let query = request.query.trim().to_string();
        if query.is_empty() {
            return Err(AppError::BadRequest("search query is required".to_string()));
        }
        let mut hits = self
            .inner
            .repository
            .rag_fts_search(&query, request.source_run_id.as_deref(), limit)
            .await?;
        if let Some(model_id) = request.embedding_model_id.as_deref() {
            let model = self.embedding_model(model_id).await?;
            let runtime_id = self.selected_embedding_runtime_id().await;
            let mut profile = profile_from_model_row(
                &self.inner.config.app_root,
                &self.inner.config.model_dir,
                &model,
                Some(&runtime_id),
            )?;
            profile.dimension = model.dimension;
            let vectors =
                generate_embeddings(profile, EmbeddingPurpose::Query, vec![query.clone()]).await?;
            if let Some(vector) = vectors.first() {
                hits.extend(
                    self.inner
                        .repository
                        .rag_vss_search(
                            vector,
                            model.dimension,
                            model_id,
                            request.source_run_id.as_deref(),
                            limit,
                        )
                        .await?,
                );
            }
        }
        Ok(HybridSearchResponse {
            query,
            files: grouped_search_hits(hits),
        })
    }

}
