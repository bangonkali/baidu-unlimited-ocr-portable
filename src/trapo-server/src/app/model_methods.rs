impl AppState {
    pub async fn models(&self) -> ModelsPayload {
        let state = self.inner.state.lock().await;
        ModelsPayload {
            provider_repo: PROVIDER_REPO_ID.to_string(),
            provider_label: PROVIDER_LABEL.to_string(),
            selected_model_id: state.selected_model_id.clone(),
            shared_mmproj_file: SHARED_MMPROJ_FILE.to_string(),
            models: model_catalog()
                .iter()
                .map(|entry| model_record(&self.inner.config.model_dir, &state, entry))
                .collect(),
            profiles: ocr_profiles(),
        }
    }

    pub async fn select_model(&self, model_id: &str) -> Result<ModelSelectRecord> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let event = {
            let mut state = self.inner.state.lock().await;
            state.selected_model_id = model_id.to_string();
            self.inner
                .repository
                .put_setting("selected_model_id", &Value::String(model_id.to_string()))?;
            serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?
        };
        self.log_info("models", format!("selected model {model_id}"))
            .await;
        self.inner.hub.publish("model.changed", event);
        self.publish_status_changed().await;
        Ok(ModelSelectRecord {
            model_id: model_id.to_string(),
            status: self.model_status(model_id).await,
        })
    }

    pub async fn start_model_download(
        &self,
        model_id: &str,
        request: ModelDownloadRequest,
    ) -> Result<ModelDownloadRecord> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        if self.model_ready(entry.model_id).await && request.force != Some(true) {
            return Ok(ModelDownloadRecord {
                model_id: model_id.to_string(),
                status: "downloaded".to_string(),
            });
        }
        {
            let mut state = self.inner.state.lock().await;
            if state
                .downloads
                .values()
                .any(|download| download.status == "downloading")
            {
                return Err(AppError::Conflict(
                    "another model download is already running".to_string(),
                ));
            }
            state.downloads.insert(
                model_id.to_string(),
                DownloadState {
                    status: "downloading".to_string(),
                    current_file: None,
                    downloaded_bytes: 0,
                    total_bytes: Some(entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES),
                    error: None,
                    started_at: Some(Instant::now()),
                    cancel_requested: false,
                    last_event_at: Some(Utc::now().to_rfc3339()),
                },
            );
        }
        self.spawn_download(model_id.to_string());
        Ok(ModelDownloadRecord {
            model_id: model_id.to_string(),
            status: "downloading".to_string(),
        })
    }

    pub async fn cancel_model_download(&self, model_id: &str) -> Result<ModelDownloadRecord> {
        if find_model(model_id).is_none() {
            return Err(AppError::BadRequest("unknown model id".to_string()));
        }
        let status = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get_mut(model_id) {
                download.cancel_requested = true;
                download.status = "cancelling".to_string();
                "cancelling".to_string()
            } else {
                "idle".to_string()
            }
        };
        Ok(ModelDownloadRecord {
            model_id: model_id.to_string(),
            status,
        })
    }

    pub async fn model_download_event(&self, model_id: &str) -> Result<ModelAssetRecord> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let state = self.inner.state.lock().await;
        Ok(model_record(&self.inner.config.model_dir, &state, entry))
    }
}
