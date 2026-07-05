const MODEL_DOWNLOAD_PROGRESS_FLUSH_MS: u64 = 250;
const MODEL_DOWNLOAD_PROGRESS_FLUSH_BYTES: u64 = 2 * 1024 * 1024;

impl AppState {
    async fn bump_download_progress(
        &self,
        download_id: &str,
        downloaded: u64,
        total: u64,
    ) -> Result<()> {
        let event = {
            let mut state = self.inner.state.lock().await;
            let now = Instant::now();
            let occurred_at = Utc::now().to_rfc3339();
            let snapshot = {
                let Some(download) = state.downloads.get_mut(download_id) else {
                    return Ok(());
                };
                download.downloaded_bytes = downloaded;
                download.total_bytes = Some(total);
                if !should_publish_download_progress(download, downloaded, now) {
                    return Ok(());
                }
                mark_download_progress_published(download, downloaded, now, occurred_at);
                download.clone()
            };
            let event = download_owner_event(&self.inner.config.model_dir, &state, &snapshot);
            drop(state);
            event
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
            let event = download_owner_event(&self.inner.config.model_dir, &state, &completed);
            drop(state);
            (event, completed)
        };
        self.record_download_event(&completed, "completed").await;
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
            let event = download_owner_event(&self.inner.config.model_dir, &state, &failed);
            drop(state);
            (event, failed)
        };
        self.record_download_event(&failed, "failed").await;
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
            let event = download_owner_event(&self.inner.config.model_dir, &state, &cancelled);
            drop(state);
            (event, cancelled)
        };
        self.record_download_event(&cancelled, "cancelled").await;
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        self.spawn_next_download().await;
    }

    async fn download_cancelled(&self, download_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        let cancelled = state
            .downloads
            .get(download_id)
            .is_some_and(|download| download.cancel_requested);
        drop(state);
        cancelled
    }

    async fn record_download_event(&self, download: &DownloadState, event_type: &str) {
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
        if let Err(error) = self.inner.repository.insert_download_event(&event).await {
            tracing::warn!(%error, download_id = %download.download_id, "failed to persist download event");
        }
    }
}

fn should_publish_download_progress(
    download: &DownloadState,
    downloaded: u64,
    now: Instant,
) -> bool {
    let Some(last_published_at) = download.last_progress_publish_at else {
        return true;
    };
    if downloaded.saturating_sub(download.last_progress_publish_bytes)
        >= MODEL_DOWNLOAD_PROGRESS_FLUSH_BYTES
    {
        return true;
    }
    now.checked_duration_since(last_published_at)
        .is_none_or(|elapsed| elapsed.as_millis() >= u128::from(MODEL_DOWNLOAD_PROGRESS_FLUSH_MS))
}

fn mark_download_progress_published(
    download: &mut DownloadState,
    downloaded: u64,
    now: Instant,
    occurred_at: String,
) {
    download.last_progress_publish_at = Some(now);
    download.last_progress_publish_bytes = downloaded;
    download.last_event_at = Some(occurred_at);
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
