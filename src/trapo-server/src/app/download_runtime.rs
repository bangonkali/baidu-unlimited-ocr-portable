use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use trapo_downloader::{
    DownloadFailure, DownloadOutcome, DownloadRequest, Downloader, DownloaderOptions,
};

impl AppState {
    fn model_ready(&self, model_id: &str) -> bool {
        find_model(model_id).is_some_and(|entry| {
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
            state.download_file(download_id).await;
        });
    }

    async fn spawn_available_downloads(&self) {
        if self.inner.shutdown.is_requested() {
            return;
        }
        let started = {
            let mut state = self.inner.state.lock().await;
            let started = start_queued_downloads(&mut state);
            let events = started
                .iter()
                .filter_map(|download| download_owner_event(&self.inner.config.model_dir, &state, download))
                .collect::<Vec<_>>();
            drop(state);
            (started, events)
        };
        for download in &started.0 {
            self.record_download_event(download, "started").await;
        }
        for event in started.1 {
            self.inner.hub.publish("model.changed", event);
        }
        for download in started.0 {
            self.spawn_download(download.download_id);
        }
    }

    async fn download_file(&self, download_id: String) {
        let Some(download) = self.download_snapshot(&download_id).await else {
            return;
        };
        if !download.force && file_is_present(&download.target_path) {
            let total = download.total_bytes;
            self.set_download_complete(
                &download_id,
                total.unwrap_or(download.downloaded_bytes),
                total,
            )
            .await;
            return;
        }
        let Some(request) = self.download_request(&download).await else {
            return;
        };
        let Some(downloader) = self.download_engine(&download_id).await else {
            return;
        };
        let outcome = self
            .run_download_transfer(downloader, request, download.cancel_flag.clone())
            .await;
        self.apply_download_outcome(&download_id, outcome).await;
    }

    async fn download_request(&self, download: &DownloadState) -> Option<DownloadRequest> {
        let temp_path = download_temp_path(&download.target_path, &download.file_id);
        match DownloadRequest::new(
            download.download_id.clone(),
            &download.source_url,
            download.target_path.clone(),
            temp_path,
            download.total_bytes,
            download.force,
        ) {
            Ok(request) => Some(request),
            Err(error) => {
                self.report_download_failure(&download.download_id, error)
                    .await;
                None
            }
        }
    }

    async fn download_engine(&self, download_id: &str) -> Option<Downloader> {
        match Downloader::new(download_options()) {
            Ok(downloader) => Some(downloader),
            Err(error) => {
                self.report_download_failure(download_id, error).await;
                None
            }
        }
    }

    async fn run_download_transfer(
        &self,
        downloader: Downloader,
        request: DownloadRequest,
        cancel_flag: Arc<AtomicBool>,
    ) -> std::result::Result<DownloadOutcome, DownloadFailure> {
        let progress_state = self.clone();
        downloader
            .download_file(request, cancel_flag, |progress| {
                let progress_state = progress_state.clone();
                async move {
                    if let Err(error) = progress_state
                        .bump_download_progress(
                            &progress.download_id,
                            progress.downloaded_bytes,
                            progress.total_bytes,
                        )
                        .await
                    {
                        tracing::warn!(%error, "failed to publish download progress");
                    }
                }
            })
            .await
    }

    async fn apply_download_outcome(
        &self,
        download_id: &str,
        outcome: std::result::Result<DownloadOutcome, DownloadFailure>,
    ) {
        match outcome {
            Ok(DownloadOutcome::Completed {
                downloaded_bytes,
                total_bytes,
            }) => {
                self.set_download_complete(download_id, downloaded_bytes, total_bytes)
                    .await;
            }
            Ok(DownloadOutcome::Cancelled {
                downloaded_bytes,
                total_bytes,
            }) => {
                self.set_download_cancelled(download_id, downloaded_bytes, total_bytes)
                    .await;
            }
            Err(error) => {
                self.report_download_failure(download_id, error).await;
            }
        }
    }

    async fn report_download_failure(&self, download_id: &str, error: DownloadFailure) {
        self.set_download_error(
            download_id,
            error.kind().as_str().to_string(),
            error.to_string(),
            Some(error.downloaded_bytes()),
            error.total_bytes(),
        )
        .await;
    }

    async fn download_snapshot(&self, download_id: &str) -> Option<DownloadState> {
        let state = self.inner.state.lock().await;
        state.downloads.get(download_id).cloned()
    }
}

fn start_queued_downloads(state: &mut WorkbenchState) -> Vec<DownloadState> {
    let active = state
        .downloads
        .values()
        .filter(|download| matches!(download.status.as_str(), "downloading" | "cancelling"))
        .count();
    let limit = usize::try_from(state.download_concurrency).unwrap_or(usize::MAX);
    let mut started = Vec::new();
    while active + started.len() < limit {
        let Some(download_id) = state.download_queue.pop_front() else {
            break;
        };
        let Some(download) = state.downloads.get_mut(&download_id) else {
            continue;
        };
        if download.status != "queued" {
            continue;
        }
        download.status = "downloading".to_string();
        download.cancel_requested = false;
        download.cancel_flag.store(false, Ordering::Relaxed);
        download.started_at = Some(Instant::now());
        download.last_event_at = Some(Utc::now().to_rfc3339());
        started.push(download.clone());
    }
    started
}

fn download_options() -> DownloaderOptions {
    let mut headers = HeaderMap::new();
    if let Some(token) = hf_token() {
        let value = format!("Bearer {token}");
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(AUTHORIZATION, value);
        }
    }
    DownloaderOptions {
        headers,
        ..DownloaderOptions::default()
    }
}

fn download_temp_path(target_path: &Path, file_id: &str) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");
    target_path.with_file_name(format!("{file_name}.{file_id}.part"))
}
