use std::{future::Future, time::Duration};

const SHUTDOWN_GRACE_MS: u64 = 5_000;
const SHUTDOWN_DRAIN_MS: u64 = 2_000;

impl AppState {
    /// Returns the cancellation token that resolves when shutdown has started.
    #[must_use]
    pub fn shutdown_token(&self) -> tokio_util::sync::CancellationToken {
        self.inner.shutdown.token()
    }

    /// Requests shutdown from an operating-system signal.
    pub async fn request_signal_shutdown(&self, source: &str) {
        let _ = self.request_shutdown(source).await;
    }

    pub(crate) async fn request_shutdown(&self, source: &str) -> Result<ShutdownPayload> {
        let record = self.inner.shutdown.request(source);
        let summary = self.cancel_work_for_shutdown().await?;
        self.log_warn(
            "server",
            format!(
                "shutdown requested by {} at {}; cancelling active work",
                record.source, record.requested_at
            ),
        );
        self.publish_status_changed().await;
        Ok(ShutdownPayload {
            active_download_count: summary.active_download_count,
            active_run_ids: summary.active_run_ids,
            grace_ms: SHUTDOWN_GRACE_MS,
            message: "Trapo is shutting down. Restart the server when you are ready.".to_string(),
            source: record.source,
            state: "shutting_down".to_string(),
        })
    }

    /// Drains background work and persistence queues after the listener stops.
    pub async fn complete_shutdown(&self) {
        if !self.inner.shutdown.is_requested() {
            return;
        }
        let source = self
            .inner
            .shutdown
            .record()
            .map_or_else(|| "unknown".to_string(), |record| record.source);
        self.log_info("server", format!("draining shutdown requested by {source}"));
        let remaining = self
            .inner
            .background_tasks
            .wait_or_abort(Duration::from_millis(SHUTDOWN_GRACE_MS))
            .await;
        if remaining > 0 {
            self.log_warn(
                "server",
                format!("aborted {remaining} background tasks after shutdown grace period"),
            );
        }
        self.inner
            .annotation_identities
            .shutdown(
                &self.inner.repository,
                Duration::from_millis(SHUTDOWN_DRAIN_MS),
            )
            .await;
        self.inner
            .hub
            .shutdown_persistence(Duration::from_millis(SHUTDOWN_DRAIN_MS))
            .await;
        if let Err(error) = self.inner.repository.checkpoint().await {
            self.log_warn("server", format!("DuckDB checkpoint failed during shutdown: {error}"));
        }
        self.inner.logger.flush();
    }

    fn spawn_background<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.inner.shutdown.is_requested() {
            return;
        }
        self.inner.background_tasks.spawn(future);
    }

    fn ensure_not_shutting_down(&self) -> Result<()> {
        if self.inner.shutdown.is_requested() {
            return Err(AppError::Conflict(
                "trapo-server is shutting down and cannot start new work".to_string(),
            ));
        }
        Ok(())
    }

    async fn ensure_no_active_ingest(&self) -> Result<()> {
        let state = self.inner.state.lock().await;
        let active = state.active_run_id.is_some()
            || state.runs.values().any(|run| run_is_active(&run.status));
        drop(state);
        if active {
            return Err(AppError::Conflict(
                "an ingest run is already queued or running".to_string(),
            ));
        }
        Ok(())
    }

    async fn cancel_work_for_shutdown(&self) -> Result<ShutdownWorkSummary> {
        let updates = self.collect_shutdown_updates().await?;
        self.persist_shutdown_updates(&updates).await?;
        self.publish_shutdown_updates(&updates);
        Ok(updates.summary)
    }

    async fn collect_shutdown_updates(&self) -> Result<ShutdownUpdates> {
        let mut state = self.inner.state.lock().await;
        let mut updates = ShutdownUpdates::default();
        collect_active_run_updates(&mut state, &mut updates)?;
        let active_run_ids = updates.summary.active_run_ids.clone();
        collect_document_updates(&mut state, &active_run_ids, &mut updates)?;
        collect_download_updates(&self.inner.config.model_dir, &mut state, &mut updates);
        drop(state);
        Ok(updates)
    }

    async fn persist_shutdown_updates(&self, updates: &ShutdownUpdates) -> Result<()> {
        for run in &updates.runs {
            self.inner.repository.upsert_run(run).await?;
        }
        for document in &updates.documents {
            self.inner.repository.upsert_document(document).await?;
        }
        for download in &updates.downloads {
            self.record_download_event(download, "cancel_requested").await;
        }
        Ok(())
    }

    fn publish_shutdown_updates(&self, updates: &ShutdownUpdates) {
        for event in &updates.run_events {
            self.inner.hub.publish("run.changed", event.clone());
        }
        for event in &updates.document_events {
            self.inner.hub.publish("document.changed", event.clone());
        }
        for event in &updates.model_events {
            self.inner.hub.publish("model.changed", event.clone());
        }
    }
}

#[derive(Default)]
struct ShutdownUpdates {
    summary: ShutdownWorkSummary,
    runs: Vec<StoredRun>,
    documents: Vec<StoredDocument>,
    downloads: Vec<DownloadState>,
    run_events: Vec<Value>,
    document_events: Vec<Value>,
    model_events: Vec<Value>,
}

#[derive(Default)]
struct ShutdownWorkSummary {
    active_download_count: u32,
    active_run_ids: Vec<String>,
}

fn collect_active_run_updates(
    state: &mut WorkbenchState,
    updates: &mut ShutdownUpdates,
) -> Result<()> {
    for run in state.runs.values_mut().filter(|run| run_is_active(&run.status)) {
        run.cancel_requested = true;
        run.status = "cancelled".to_string();
        updates.summary.active_run_ids.push(run.run_id.clone());
        updates.run_events.push(serde_json::to_value(run_record(run))?);
        updates.runs.push(stored_run(run));
    }
    if updates
        .summary
        .active_run_ids
        .iter()
        .any(|run_id| state.active_run_id.as_deref() == Some(run_id))
    {
        state.active_run_id = None;
    }
    Ok(())
}

fn collect_document_updates(
    state: &mut WorkbenchState,
    active_run_ids: &[String],
    updates: &mut ShutdownUpdates,
) -> Result<()> {
    let active_hashes: Vec<String> = state
        .runs
        .values()
        .filter(|run| active_run_ids.iter().any(|run_id| run_id == &run.run_id))
        .flat_map(|run| run.file_hashes.clone())
        .collect();
    for file_hash in active_hashes {
        if let Some(document) = state.documents.get_mut(&file_hash) {
            if document_is_active(&document.status) {
                document.status = "cancelled".to_string();
            }
            updates.documents.push(stored_document(document));
            updates
                .document_events
                .push(serde_json::to_value(document_summary(document))?);
        }
    }
    Ok(())
}

fn collect_download_updates(
    model_dir: &Path,
    state: &mut WorkbenchState,
    updates: &mut ShutdownUpdates,
) {
    state.download_queue.clear();
    let active_count = state
        .downloads
        .values()
        .filter(|download| is_active_download_status(&download.status))
        .count();
    updates.summary.active_download_count = usize_to_u32_saturating(active_count);
    let active_download_ids: Vec<String> = state
        .downloads
        .iter()
        .filter(|(_, download)| is_active_download_status(&download.status))
        .map(|(download_id, _)| download_id.clone())
        .collect();
    for download_id in active_download_ids {
        collect_download_update(model_dir, state, &download_id, updates);
    }
}

fn collect_download_update(
    model_dir: &Path,
    state: &mut WorkbenchState,
    download_id: &str,
    updates: &mut ShutdownUpdates,
) {
    let Some(download) = state.downloads.get_mut(download_id) else {
        return;
    };
    download.cancel_requested = true;
    download.status = if download.status == "queued" {
        "cancelled".to_string()
    } else {
        "cancelling".to_string()
    };
    download.last_event_at = Some(Utc::now().to_rfc3339());
    let snapshot = download.clone();
    if let Some(event) = download_owner_event(model_dir, state, &snapshot) {
        updates.model_events.push(event);
    }
    updates.downloads.push(snapshot);
}

fn run_is_active(status: &str) -> bool {
    matches!(status, "queued" | "running")
}

fn document_is_active(status: &str) -> bool {
    matches!(status, "queued" | "running" | "rendering")
}
