impl AppState {
    async fn bump_download_progress(
        &self,
        download_id: &str,
        downloaded: u64,
        total: u64,
    ) -> Result<()> {
        let event = {
            let mut state = self.inner.state.lock().await;
            let snapshot = {
                let Some(download) = state.downloads.get_mut(download_id) else {
                    return Ok(());
                };
                download.downloaded_bytes = downloaded;
                download.total_bytes = Some(total);
                download.last_event_at = Some(Utc::now().to_rfc3339());
                download.clone()
            };
            download_owner_event(&self.inner.config.model_dir, &state, &snapshot)
        };
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        Ok(())
    }

    async fn set_download_complete(&self, download_id: &str) -> Result<()> {
        let (event, completed) = {
            let mut state = self.inner.state.lock().await;
            let completed = {
                let Some(download) = state.downloads.get_mut(download_id) else {
                    return Ok(());
                };
                download.status = "downloaded".to_string();
                download.downloaded_bytes = download.total_bytes.unwrap_or(download.downloaded_bytes);
                download.error = None;
                download.last_event_at = Some(Utc::now().to_rfc3339());
                download.clone()
            };
            (
                download_owner_event(&self.inner.config.model_dir, &state, &completed),
                completed,
            )
        };
        self.record_download_event(&completed, "completed");
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
        Ok(())
    }

    async fn set_download_error(&self, download_id: &str, error: String) {
        let (event, failed) = {
            let mut state = self.inner.state.lock().await;
            let failed = {
                let Some(download) = state.downloads.get_mut(download_id) else {
                    return;
                };
                download.status = "failed".to_string();
                download.error = Some(error);
                download.last_event_at = Some(Utc::now().to_rfc3339());
                download.clone()
            };
            (
                download_owner_event(&self.inner.config.model_dir, &state, &failed),
                failed,
            )
        };
        self.record_download_event(&failed, "failed");
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
    }

    async fn set_download_cancelled(&self, download_id: &str) {
        let (event, cancelled) = {
            let mut state = self.inner.state.lock().await;
            let cancelled = {
                let Some(download) = state.downloads.get_mut(download_id) else {
                    return;
                };
                download.status = "cancelled".to_string();
                download.last_event_at = Some(Utc::now().to_rfc3339());
                download.clone()
            };
            (
                download_owner_event(&self.inner.config.model_dir, &state, &cancelled),
                cancelled,
            )
        };
        self.record_download_event(&cancelled, "cancelled");
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
    }

    async fn download_cancelled(&self, download_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        state
            .downloads
            .get(download_id)
            .is_some_and(|download| download.cancel_requested)
    }

    fn record_download_event(&self, download: &DownloadState, event_type: &str) {
        let event = DownloadEventInsert {
            event_id: uuid::Uuid::new_v4().to_string(),
            download_id: download.download_id.clone(),
            owner_kind: download.owner_kind.clone(),
            owner_id: download.owner_id.clone(),
            file_id: download.file_id.clone(),
            file_name: download.file_name.clone(),
            target_path: download.target_path.to_string_lossy().to_string(),
            source_url: download.source_url.clone(),
            event_type: event_type.to_string(),
            status: download.status.clone(),
            downloaded_bytes: download.downloaded_bytes,
            total_bytes: download.total_bytes,
            error: download.error.clone(),
            created_at: Utc::now().to_rfc3339(),
        };
        if let Err(error) = self.inner.repository.insert_download_event(&event) {
            tracing::warn!(%error, download_id = %download.download_id, "failed to persist download event");
        }
    }
}

fn download_owner_event(
    model_dir: &Path,
    state: &WorkbenchState,
    download: &DownloadState,
) -> Option<Value> {
    if download.owner_kind != "model" {
        return None;
    }
    find_model(&download.owner_id)
        .and_then(|entry| serde_json::to_value(model_record(model_dir, state, entry)).ok())
}
