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
        let (status, event) = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get(model_id)
                && matches!(
                    download.status.as_str(),
                    "downloading" | "queued" | "cancelling"
                )
            {
                return Ok(ModelDownloadRecord {
                    model_id: model_id.to_string(),
                    status: download.status.clone(),
                });
            }
            let active = state
                .downloads
                .values()
                .any(|download| matches!(download.status.as_str(), "downloading" | "cancelling"));
            let status = if active { "queued" } else { "downloading" }.to_string();
            if active && !state.download_queue.iter().any(|item| item == model_id) {
                state.download_queue.push_back(model_id.to_string());
            }
            state.downloads.insert(
                model_id.to_string(),
                DownloadState {
                    status: status.clone(),
                    current_file: None,
                    downloaded_bytes: 0,
                    total_bytes: Some(entry.model_size_bytes + SHARED_MMPROJ_SIZE_BYTES),
                    error: None,
                    started_at: (!active).then(Instant::now),
                    cancel_requested: false,
                    last_event_at: Some(Utc::now().to_rfc3339()),
                },
            );
            (
                status,
                serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?,
            )
        };
        self.inner.hub.publish("model.changed", event);
        if status == "downloading" {
            self.spawn_download(model_id.to_string());
        }
        Ok(ModelDownloadRecord {
            model_id: model_id.to_string(),
            status,
        })
    }

    pub async fn cancel_model_download(&self, model_id: &str) -> Result<ModelDownloadRecord> {
        if find_model(model_id).is_none() {
            return Err(AppError::BadRequest("unknown model id".to_string()));
        }
        let (status, event) = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get_mut(model_id) {
                download.cancel_requested = true;
                download.status = if download.status == "queued" {
                    "cancelled".to_string()
                } else {
                    "cancelling".to_string()
                };
                download.last_event_at = Some(Utc::now().to_rfc3339());
                let entry = find_model(model_id)
                    .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
                (
                    download.status.clone(),
                    Some(serde_json::to_value(model_record(
                        &self.inner.config.model_dir,
                        &state,
                        entry,
                    ))?),
                )
            } else {
                ("idle".to_string(), None)
            }
        };
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
        Ok(ModelDownloadRecord {
            model_id: model_id.to_string(),
            status,
        })
    }

    pub async fn model_download_event(&self, model_id: &str) -> Result<ModelDownloadEvent> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let state = self.inner.state.lock().await;
        let record = model_record(&self.inner.config.model_dir, &state, entry);
        Ok(ModelDownloadEvent {
            phase: record.status.clone(),
            message: record.status_message.clone().unwrap_or_default(),
            model: record,
        })
    }
}
