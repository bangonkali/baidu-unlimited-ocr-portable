impl AppState {
    async fn set_download_file(&self, model_id: &str, file_name: &str, total: u64) -> Result<()> {
        let event = {
            let mut state = self.inner.state.lock().await;
            let Some(download) = state.downloads.get_mut(model_id) else {
                return Ok(());
            };
            download.current_file = Some(file_name.to_string());
            download.total_bytes = Some(total);
            download.last_event_at = Some(Utc::now().to_rfc3339());
            let entry = find_model(model_id)
                .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
            serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?
        };
        self.inner.hub.publish("model.changed", event);
        self.spawn_next_download().await;
        Ok(())
    }

    async fn bump_download_progress(
        &self,
        model_id: &str,
        file_name: &str,
        downloaded: u64,
        total: u64,
    ) -> Result<()> {
        let event = {
            let mut state = self.inner.state.lock().await;
            let Some(download) = state.downloads.get_mut(model_id) else {
                return Ok(());
            };
            download.current_file = Some(file_name.to_string());
            download.downloaded_bytes = downloaded;
            download.total_bytes = Some(total);
            download.last_event_at = Some(Utc::now().to_rfc3339());
            let entry = find_model(model_id)
                .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
            serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?
        };
        self.inner.hub.publish("model.changed", event);
        Ok(())
    }

    async fn set_download_complete(&self, model_id: &str) -> Result<()> {
        let event = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get_mut(model_id) {
                download.status = "downloaded".to_string();
                download.current_file = None;
                download.error = None;
                download.last_event_at = Some(Utc::now().to_rfc3339());
            }
            let entry = find_model(model_id)
                .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
            serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))?
        };
        self.inner.hub.publish("model.changed", event);
        Ok(())
    }

    async fn set_download_error(&self, model_id: &str, error: String) {
        let event = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get_mut(model_id) {
                download.status = "failed".to_string();
                download.error = Some(error);
                download.last_event_at = Some(Utc::now().to_rfc3339());
            }
            find_model(model_id).and_then(|entry| {
                serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry)).ok()
            })
        };
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
    }

    async fn set_download_cancelled(&self, model_id: &str) {
        let event = {
            let mut state = self.inner.state.lock().await;
            if let Some(download) = state.downloads.get_mut(model_id) {
                download.status = "cancelled".to_string();
                download.last_event_at = Some(Utc::now().to_rfc3339());
            }
            find_model(model_id).and_then(|entry| {
                serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry)).ok()
            })
        };
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
    }

    async fn download_cancelled(&self, model_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        state
            .downloads
            .get(model_id)
            .is_some_and(|download| download.cancel_requested)
    }
}
