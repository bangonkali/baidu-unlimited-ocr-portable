use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

impl AppState {
    fn model_ready(&self, model_id: &str) -> bool {
        find_model(model_id)
            .is_some_and(|entry| {
                model_download_targets(&self.inner.config.model_dir, entry)
                    .iter()
                    .all(|target| file_is_present(&target.target_path))
            })
    }

    async fn model_status(&self, model_id: &str) -> String {
        let Some(entry) = find_model(model_id) else {
            return "missing".to_string();
        };
        let state = self.inner.state.lock().await;
        model_record(&self.inner.config.model_dir, &state, entry).status
    }

    fn spawn_download(&self, download_id: String) {
        let state = self.clone();
        self.spawn_background(async move {
            if let Err(error) = state.download_file(download_id.clone()).await {
                state
                    .set_download_error(&download_id, format!("download failed: {error}"))
                    .await;
            }
        });
    }

    async fn spawn_next_download(&self) {
        if self.inner.shutdown.is_requested() {
            return;
        }
        let (next_download_id, event, started) = {
            let mut state = self.inner.state.lock().await;
            let active = state
                .downloads
                .values()
                .any(|download| matches!(download.status.as_str(), "downloading" | "cancelling"));
            if active {
                return;
            }
            let mut started = None;
            while let Some(download_id) = state.download_queue.pop_front() {
                let Some(download) = state.downloads.get_mut(&download_id) else {
                    continue;
                };
                if download.status != "queued" {
                    continue;
                }
                download.status = "downloading".to_string();
                download.started_at = Some(Instant::now());
                download.last_event_at = Some(Utc::now().to_rfc3339());
                started = Some(download.clone());
                break;
            }
            let event = started.as_ref().and_then(|download| {
                download_owner_event(&self.inner.config.model_dir, &state, download)
            });
            let next_download_id = started
                .as_ref()
                .map(|download| download.download_id.clone());
            drop(state);
            (next_download_id, event, started)
        };
        if let Some(download) = started.as_ref() {
            self.record_download_event(download, "started").await;
        }
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        if let Some(download_id) = next_download_id {
            self.spawn_download(download_id);
        }
    }

    async fn download_file(&self, download_id: String) -> Result<()> {
        let download = {
            let state = self.inner.state.lock().await;
            state
                .downloads
                .get(&download_id)
                .cloned()
                .ok_or_else(|| AppError::BadRequest("unknown download id".to_string()))?
        };
        if !download.force && file_is_present(&download.target_path) {
            self.set_download_complete(&download_id).await?;
            return Ok(());
        }
        if let Some(parent) = download.target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let client = reqwest::Client::new();
        let mut request = client.get(&download.source_url);
        if let Some(token) = hf_token() {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .await
            .map_err(|error| AppError::Internal(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let mut stream = response.bytes_stream();
        let temp = download_temp_path(&download.target_path, &download.file_id);
        let mut file = tokio::fs::File::create(&temp).await?;
        let mut downloaded = 0_u64;
        while let Some(chunk) = stream.next().await {
            if self.download_cancelled(&download_id).await {
                let _ = tokio::fs::remove_file(&temp).await;
                self.set_download_cancelled(&download_id).await;
                return Ok(());
            }
            let chunk = chunk.map_err(|error| AppError::Internal(error.to_string()))?;
            file.write_all(&chunk).await?;
            downloaded = downloaded.saturating_add(usize_to_u64_saturating(chunk.len()));
            self.bump_download_progress(&download_id, downloaded, download.total_bytes.unwrap_or(0))
                .await?;
        }
        file.flush().await?;
        if file_is_present(&download.target_path) {
            tokio::fs::remove_file(&download.target_path).await?;
        }
        tokio::fs::rename(&temp, &download.target_path).await?;
        self.set_download_complete(&download_id).await
    }
}

fn download_temp_path(target_path: &Path, file_id: &str) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");
    target_path.with_file_name(format!("{file_name}.{file_id}.part"))
}
