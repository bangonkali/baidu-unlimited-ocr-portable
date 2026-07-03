impl AppState {
    async fn model_ready(&self, model_id: &str) -> bool {
        find_model(model_id)
            .map(|entry| {
                self.inner.config.model_dir.join(entry.model_file).is_file()
                    && self
                        .inner
                        .config
                        .model_dir
                        .join(SHARED_MMPROJ_FILE)
                        .is_file()
            })
            .unwrap_or(false)
    }

    async fn model_status(&self, model_id: &str) -> String {
        if self.model_ready(model_id).await {
            return "downloaded".to_string();
        }
        let state = self.inner.state.lock().await;
        state
            .downloads
            .get(model_id)
            .map(|download| download.status.clone())
            .unwrap_or_else(|| "missing".to_string())
    }

    fn spawn_download(&self, model_id: String) {
        let state = self.clone();
        tokio::spawn(async move {
            if let Err(error) = state.download_model(model_id.clone()).await {
                state
                    .set_download_error(&model_id, format!("download failed: {error}"))
                    .await;
            }
        });
    }

    async fn spawn_next_download(&self) {
        let (next_model_id, event) = {
            let mut state = self.inner.state.lock().await;
            let active = state
                .downloads
                .values()
                .any(|download| matches!(download.status.as_str(), "downloading" | "cancelling"));
            if active {
                return;
            }
            let mut next_model_id = None;
            while let Some(model_id) = state.download_queue.pop_front() {
                let Some(download) = state.downloads.get_mut(&model_id) else {
                    continue;
                };
                if download.status != "queued" {
                    continue;
                }
                download.status = "downloading".to_string();
                download.started_at = Some(Instant::now());
                download.last_event_at = Some(Utc::now().to_rfc3339());
                next_model_id = Some(model_id);
                break;
            }
            let event = next_model_id.as_deref().and_then(|model_id| {
                find_model(model_id).and_then(|entry| {
                    serde_json::to_value(model_record(&self.inner.config.model_dir, &state, entry))
                        .ok()
                })
            });
            (next_model_id, event)
        };
        if let Some(event) = event {
            self.inner.hub.publish("model.changed", event);
        }
        if let Some(model_id) = next_model_id {
            self.spawn_download(model_id);
        }
    }

    async fn download_model(&self, model_id: String) -> Result<()> {
        let entry = find_model(&model_id)
            .ok_or_else(|| AppError::BadRequest("unknown model id".to_string()))?;
        let files = [
            ("model", entry.model_file, entry.model_size_bytes),
            ("mmproj", SHARED_MMPROJ_FILE, SHARED_MMPROJ_SIZE_BYTES),
        ];
        let client = reqwest::Client::new();
        for (file_id, file_name, expected_size) in files {
            if self.download_cancelled(&model_id).await {
                self.set_download_cancelled(&model_id).await;
                return Ok(());
            }
            let target = self.inner.config.model_dir.join(file_name);
            if target.is_file()
                && target
                    .metadata()
                    .map(|metadata| metadata.len())
                    .unwrap_or(0)
                    > 0
            {
                self.bump_download_progress(&model_id, file_name, expected_size, expected_size)
                    .await?;
                continue;
            }
            let url = format!(
                "https://huggingface.co/{}/resolve/{}/{}",
                PROVIDER_REPO_ID, PROVIDER_REVISION, file_name
            );
            self.set_download_file(&model_id, file_name, expected_size)
                .await?;
            let mut request = client.get(&url);
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
            let temp = target.with_extension(format!("{file_id}.part"));
            let mut file = tokio::fs::File::create(&temp).await?;
            let mut downloaded = 0_u64;
            use futures_util::StreamExt;
            use tokio::io::AsyncWriteExt;
            while let Some(chunk) = stream.next().await {
                if self.download_cancelled(&model_id).await {
                    let _ = tokio::fs::remove_file(&temp).await;
                    self.set_download_cancelled(&model_id).await;
                    return Ok(());
                }
                let chunk = chunk.map_err(|error| AppError::Internal(error.to_string()))?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                self.bump_download_progress(&model_id, file_name, downloaded, expected_size)
                    .await?;
            }
            file.flush().await?;
            tokio::fs::rename(&temp, &target).await?;
        }
        self.set_download_complete(&model_id).await
    }
}
