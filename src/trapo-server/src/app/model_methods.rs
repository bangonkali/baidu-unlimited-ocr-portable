impl AppState {
    pub(crate) async fn models(&self) -> ModelsPayload {
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

    pub(crate) async fn select_model(&self, model_id: &str) -> Result<ModelSelectRecord> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let event = {
            let mut state = self.inner.state.lock().await;
            state.selected_model_id = model_id.to_string();
            let event =
                serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?;
            drop(state);
            event
        };
        self.inner
            .repository
            .put_setting("selected_model_id", &Value::String(model_id.to_string()))
            .await?;
        self.log_info("models", format!("selected model {model_id}"));
        self.inner.hub.publish("model.changed", event);
        self.publish_status_changed().await;
        Ok(ModelSelectRecord {
            model_id: model_id.to_string(),
            status: self.model_status(model_id).await,
        })
    }

    pub(crate) async fn start_model_download(
        &self,
        model_id: &str,
        request: ModelDownloadRequest,
    ) -> Result<ModelDownloadRecord> {
        self.ensure_not_shutting_down()?;
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let force = request.force == Some(true);
        if self.model_ready(entry.model_id) && !force {
            return Ok(ModelDownloadRecord {
                model_id: model_id.to_string(),
                status: "downloaded".to_string(),
            });
        }
        let targets = model_download_targets(&self.inner.config.model_dir, entry);
        let (status, event, should_spawn_next) = {
            let mut state = self.inner.state.lock().await;
            let active = state
                .downloads
                .values()
                .any(|download| matches!(download.status.as_str(), "downloading" | "cancelling"));
            let queued_any = queue_download_targets(&mut state, targets, force);
            let record = model_record(&self.inner.config.model_dir, &state, entry);
            let status = record.status.clone();
            let event = serde_json::to_value(record)?;
            drop(state);
            (status, event, queued_any && !active)
        };
        self.inner.hub.publish("model.changed", event);
        if should_spawn_next {
            self.spawn_next_download().await;
        }
        Ok(ModelDownloadRecord {
            model_id: model_id.to_string(),
            status,
        })
    }

    pub(crate) async fn cancel_model_download(&self, model_id: &str) -> Result<ModelDownloadRecord> {
        if find_model(model_id).is_none() {
            return Err(AppError::BadRequest("unknown model id".to_string()));
        }
        let (status, event) = {
            let mut state = self.inner.state.lock().await;
            let mut touched = false;
            for download in state
                .downloads
                .values_mut()
                .filter(|download| {
                    download.owner_kind == "model"
                        && download.owner_id == model_id
                        && is_active_download_status(&download.status)
                })
            {
                download.cancel_requested = true;
                download.status = if download.status == "queued" {
                    "cancelled".to_string()
                } else {
                    "cancelling".to_string()
                };
                download.last_event_at = Some(Utc::now().to_rfc3339());
                touched = true;
            }
            if touched {
                let entry = find_model(model_id)
                    .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
                let record = model_record(&self.inner.config.model_dir, &state, entry);
                let status = record.status.clone();
                let event = serde_json::to_value(record)?;
                drop(state);
                (status, Some(event))
            } else {
                drop(state);
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

    pub(crate) async fn model_download_event(&self, model_id: &str) -> Result<ModelDownloadEvent> {
        let entry = find_model(model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let record = {
            let state = self.inner.state.lock().await;
            let record = model_record(&self.inner.config.model_dir, &state, entry);
            drop(state);
            record
        };
        Ok(ModelDownloadEvent {
            phase: record.status.clone(),
            message: record.status_message.clone().unwrap_or_default(),
            model: record,
        })
    }
}

fn queue_download_targets(
    state: &mut WorkbenchState,
    targets: Vec<DownloadTarget>,
    force: bool,
) -> bool {
    let mut queued_any = false;
    for target in targets {
        if !force && file_is_present(&target.target_path) {
            continue;
        }
        if download_target_is_active(state, &target) {
            queued_any = true;
            continue;
        }
        let download_id = new_persistence_id();
        state.download_queue.push_back(download_id.clone());
        state
            .downloads
            .insert(download_id.clone(), queued_download_state(download_id, target, force));
        queued_any = true;
    }
    queued_any
}

fn download_target_is_active(state: &WorkbenchState, target: &DownloadTarget) -> bool {
    state.downloads.values().any(|download| {
        download.download_key == target.download_key && is_active_download_status(&download.status)
    })
}

fn queued_download_state(
    download_id: String,
    target: DownloadTarget,
    force: bool,
) -> DownloadState {
    DownloadState {
        download_id,
        download_key: target.download_key,
        owner_kind: target.owner_kind,
        owner_id: target.owner_id,
        file_id: target.file_id,
        file_name: target.file_name,
        source_url: target.source_url,
        target_path: target.target_path,
        force,
        status: "queued".to_string(),
        downloaded_bytes: 0,
        total_bytes: Some(target.total_bytes),
        error: None,
        started_at: None,
        last_progress_publish_at: None,
        last_progress_publish_bytes: 0,
        cancel_requested: false,
        last_event_at: Some(Utc::now().to_rfc3339()),
    }
}
