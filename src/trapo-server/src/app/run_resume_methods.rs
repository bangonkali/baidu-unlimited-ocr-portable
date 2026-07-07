impl AppState {
    pub(crate) async fn resume_run(&self, run_id: &str) -> Result<IngestStartResponse> {
        self.ensure_not_shutting_down()?;
        self.ensure_no_active_pipeline_task().await?;
        let completed_pages = self.inner.repository.completed_run_pages(run_id).await?;
        let completed_page_set = completed_page_set(completed_pages);
        let prepared = self.prepare_resume_run(run_id, &completed_page_set).await?;
        self.inner.repository.upsert_run(&prepared.run_to_store).await?;
        for document in &prepared.stored_documents {
            self.inner.repository.upsert_document(document).await?;
        }
        self.log_info("ingest", format!("resume requested for run {run_id}"));
        self.inner
            .hub
            .publish("run.changed", serde_json::to_value(&prepared.run_record)?);
        for event in &prepared.document_events {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(event)?);
        }
        self.publish_status_changed().await;
        let replay_since_sequence = self.inner.hub.last_sequence();
        self.spawn_ingest(IngestExecution {
            completed_pages: completed_page_set,
            embedding_after_ingest: false,
            embedding_dimension: None,
            embedding_model_id: None,
            engine_configs: prepared.engine_configs,
            files: prepared.files,
            run_id: run_id.to_string(),
            text_index_after_ingest: false,
        });
        Ok(IngestStartResponse {
            run: self.get_run(run_id).await?,
            documents: prepared.document_events,
            replay_since_sequence,
        })
    }

    async fn prepare_resume_run(
        &self,
        run_id: &str,
        completed_pages: &BTreeSet<(String, u32)>,
    ) -> Result<PreparedResumeRun> {
        let mut state = self.inner.state.lock().await;
        let active =
            state.active_run_id.is_some() || state.runs.values().any(|run| run_is_active(&run.status));
        if active {
            return Err(AppError::Conflict(
                "an ingest run is already queued or running".to_string(),
            ));
        }
        let Some(existing) = state.runs.get(run_id) else {
            return Err(AppError::NotFound("run not found".to_string()));
        };
        if existing.completion_manifest.is_some() {
            return Err(AppError::Conflict(
                "completed runs cannot be resumed; restart from the ingest start page".to_string(),
            ));
        }
        let file_hashes = existing.file_hashes.clone();
        if file_hashes.is_empty() {
            return Err(AppError::Conflict(
                "run has no persisted document membership to resume".to_string(),
            ));
        }

        let mut files = Vec::new();
        let mut stored_documents = Vec::new();
        let mut document_events = Vec::new();
        let mut total_pages = 0_u32;
        for file_hash in &file_hashes {
            let Some(document) = state.documents.get_mut(file_hash) else {
                return Err(AppError::Conflict(format!(
                    "run document is missing from local persistence: {file_hash}"
                )));
            };
            total_pages = total_pages.saturating_add(document.page_count.max(1));
            if document_complete_for_run(document, completed_pages) {
                document.status = "completed".to_string();
            } else {
                document.status = "queued".to_string();
                document.error = None;
                files.push(discovered_file_from_document(document));
            }
            stored_documents.push(stored_document(document));
            document_events.push(document_summary(document));
        }

        let completed_count = usize_to_u32_saturating(completed_pages.len()).min(total_pages);
        let selected_runtime_id = state.selected_runtime_id.clone();
        let Some(run) = state.runs.get_mut(run_id) else {
            return Err(AppError::NotFound("run not found".to_string()));
        };
        run.status = "queued".to_string();
        run.cancel_requested = false;
        run.error = None;
        run.processed_pages = completed_count;
        run.total_pages = total_pages;
        run.current_page = if completed_count == 0 {
            None
        } else {
            Some(completed_count)
        };
        if run.runtime_id.is_empty() {
            run.runtime_id = selected_runtime_id;
        }
        let run_to_store = stored_run(run);
        let run_record = run_record(run);
        let engine_configs = run.engine_configs.clone();
        state.active_run_id = Some(run_id.to_string());
        drop(state);

        Ok(PreparedResumeRun {
            document_events,
            engine_configs,
            files,
            run_record,
            run_to_store,
            stored_documents,
        })
    }

    async fn persist_run_completion_manifest(&self, run_id: &str) -> Result<()> {
        let manifest = {
            let state = self.inner.state.lock().await;
            let run = state
                .runs
                .get(run_id)
                .ok_or_else(|| AppError::NotFound("run not found".to_string()))?;
            let manifest = StoredRunCompletionManifest {
                completed_at: Utc::now().to_rfc3339(),
                engine_id: run.engine_id.clone(),
                file_count: usize_to_u32_saturating(run.file_hashes.len()),
                model_id: run.model_id.clone(),
                page_count: run.total_pages,
                processed_pages: run.processed_pages,
                profile_id: run.profile_id.clone(),
                queued_files: run.queued_files,
                root_path: run.root_path.clone(),
                run_id: run.run_id.clone(),
                runtime_id: run.runtime_id.clone(),
                status: run.status.clone(),
                summary: json!({
                    "file_hashes": run.file_hashes.clone(),
                    "processed_pages": run.processed_pages,
                    "total_pages": run.total_pages
                }),
                total_pages: run.total_pages,
            };
            drop(state);
            manifest
        };
        self.inner
            .repository
            .upsert_run_completion_manifest(&manifest)
            .await?;
        let event = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return Err(AppError::NotFound("run not found".to_string()));
            };
            run.completion_manifest = Some(manifest);
            let event = run_record(run);
            drop(state);
            event
        };
        self.inner
            .hub
            .publish("run.changed", serde_json::to_value(event)?);
        Ok(())
    }
}

struct PreparedResumeRun {
    document_events: Vec<DocumentSummary>,
    engine_configs: Vec<RunEngineConfigState>,
    files: Vec<DiscoveredFile>,
    run_record: IngestRunRecord,
    run_to_store: StoredRun,
    stored_documents: Vec<StoredDocument>,
}

fn completed_page_set(completed_pages: Vec<CompletedRunPage>) -> BTreeSet<(String, u32)> {
    completed_pages
        .into_iter()
        .map(|page| (page.file_hash, page.page_no))
        .collect()
}

fn discovered_file_from_document(document: &DocumentState) -> DiscoveredFile {
    DiscoveredFile {
        absolute_path: document.absolute_path.clone(),
        relative_path: document.relative_path.clone(),
        size_bytes: document.size_bytes,
    }
}

fn document_complete_for_run(
    document: &DocumentState,
    completed_pages: &BTreeSet<(String, u32)>,
) -> bool {
    let page_count = document.page_count.max(1);
    (1..=page_count).all(|page_no| completed_pages.contains(&(document.file_hash.clone(), page_no)))
}
