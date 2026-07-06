impl AppState {
    pub(crate) async fn start_ingest(
        &self,
        request: IngestStartRequest,
    ) -> Result<IngestStartResponse> {
        self.ensure_not_shutting_down()?;
        self.ensure_no_active_ingest().await?;
        self.ensure_no_active_pipeline_task().await?;
        Self::validate_start_ingest_request(&request)?;
        let root = PathBuf::from(&request.root_path);
        let files = discover_supported_files(&root)?;
        let run_id = now_id();
        let (profile_id, model_id, runtime_id) = self.resolve_ingest_selection(&request).await?;
        let prepared = self
            .prepare_ingest_run(IngestPrepareInput {
                request: &request,
                root: &root,
                files: &files,
                run_id: &run_id,
                profile_id: &profile_id,
                model_id: &model_id,
                runtime_id: &runtime_id,
            })
            .await?;

        self.persist_prepared_ingest_run(PersistPreparedIngestInput {
            request: &request,
            run_id: &run_id,
            profile_id: &profile_id,
            model_id: &model_id,
            file_count: files.len(),
            prepared: &prepared,
        })
        .await?;
        self.publish_status_changed().await;
        let replay_since_sequence = self.inner.hub.last_sequence();
        self.spawn_ingest(IngestExecution {
            completed_pages: BTreeSet::new(),
            embedding_after_ingest: request.embedding_after_ingest == Some(true),
            embedding_dimension: request.embedding_dimension,
            embedding_model_id: request.embedding_model_id.clone(),
            files,
            model_id,
            profile_id,
            run_id: run_id.clone(),
            runtime_id,
            text_index_after_ingest: request.text_index_after_ingest == Some(true),
        });
        Ok(IngestStartResponse {
            run: self.get_run(&run_id).await?,
            documents: prepared.document_events,
            replay_since_sequence,
        })
    }

    fn validate_start_ingest_request(request: &IngestStartRequest) -> Result<()> {
        if request.embedding_after_ingest != Some(true)
            || !request
                .embedding_model_id
                .as_deref()
                .unwrap_or_default()
                .is_empty()
        {
            return Ok(());
        }
        Err(AppError::BadRequest(
            "embedding_model_id is required when embedding_after_ingest is enabled".to_string(),
        ))
    }

    async fn persist_prepared_ingest_run(
        &self,
        input: PersistPreparedIngestInput<'_>,
    ) -> Result<()> {
        for document in &input.prepared.stored_documents {
            self.inner.repository.upsert_document(document).await?;
        }
        for (file_hash, metadata) in &input.prepared.diagnostics {
            self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                run_id: input.run_id,
                file_hash,
                page_no: None,
                phase: "render",
                model: input.model_id,
                profile: input.profile_id,
                metadata: metadata.clone(),
            });
        }
        self.inner
            .repository
            .upsert_run(&input.prepared.run_to_store)
            .await?;
        self.inner
            .repository
            .replace_run_documents(input.run_id, &input.prepared.run_file_hashes)
            .await?;
        self.log_info(
            "ingest",
            format!(
                "scan requested for {} found {} supported files",
                input.request.root_path, input.file_count
            ),
        );
        self.inner.hub.publish(
            "run.changed",
            serde_json::to_value(&input.prepared.run_record)?,
        );
        for event in &input.prepared.document_events {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(event)?);
        }
        Ok(())
    }

    async fn resolve_ingest_selection(
        &self,
        request: &IngestStartRequest,
    ) -> Result<(String, String, String)> {
        let state = self.inner.state.lock().await;
        let profile_id = request
            .profile_id
            .clone()
            .unwrap_or_else(|| state.selected_profile_id.clone());
        let model_id = request
            .model_id
            .clone()
            .unwrap_or_else(|| state.selected_model_id.clone());
        let runtime_id = state.selected_runtime_id.clone();
        let runtime_id = request
            .runtime_id
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or(runtime_id);
        let runtime_selectable = state
            .runtime_variants
            .iter()
            .any(|item| item.runtime_id == runtime_id && item.selectable);
        drop(state);

        if find_profile(&profile_id).is_none() {
            return Err(AppError::BadRequest(format!(
                "unknown OCR profile: {profile_id}"
            )));
        }
        let Some(model) = find_model(&model_id) else {
            return Err(AppError::BadRequest(format!("unknown model id: {model_id}")));
        };
        if model.model_kind != "ocr" {
            return Err(AppError::BadRequest(format!(
                "model is not an OCR model: {model_id}"
            )));
        }
        if !runtime_selectable {
            return Err(AppError::BadRequest(format!(
                "runtime is not supported on this device or is not installed: {runtime_id}"
            )));
        }
        Ok((profile_id, model_id, runtime_id))
    }

    async fn prepare_ingest_run(&self, input: IngestPrepareInput<'_>) -> Result<PreparedIngestRun> {
        let file_count = checked_file_count(input.files.len())?;
        let mut state = self.inner.state.lock().await;
        let mut run = RunState {
            run_id: input.run_id.to_string(),
            root_path: input.request.root_path.clone(),
            status: if input.files.is_empty() {
                "completed".to_string()
            } else {
                "queued".to_string()
            },
            queued_files: file_count,
            processed_pages: 0,
            total_pages: file_count,
            current_page: None,
            profile_id: input.profile_id.to_string(),
            engine_id: input
                .request
                .engine_id
                .clone()
                .unwrap_or_else(|| ENGINE_ID.to_string()),
            model_id: input.model_id.to_string(),
            runtime_id: input.runtime_id.to_string(),
            error: None,
            cancel_requested: false,
            file_hashes: Vec::new(),
            completion_manifest: None,
        };
        let mut stored_documents = Vec::new();
        let mut diagnostics = Vec::new();
        let mut document_events = Vec::new();
        for file in input.files {
            let document = document_from_file(file, input.root); // skylos: ignore[SKY-D215] files come from discover_supported_files under validate_trusted_root().
            run.file_hashes.push(document.file_hash.clone());
            stored_documents.push(stored_document(&document));
            diagnostics.push((
                document.file_hash.clone(),
                json!({
                    "relative_path": generic_path(&document.relative_path),
                    "extension": document.extension.clone(),
                    "size_bytes": document.size_bytes
                }),
            ));
            document_events.push(document_summary(&document)); // skylos: ignore[SKY-D215] event contains metadata for a validated local document.
            state.documents.insert(document.file_hash.clone(), document);
        }
        let run_record = run_record(&run);
        let run_to_store = stored_run(&run);
        let run_file_hashes = run.file_hashes.clone();
        state.active_run_id = Some(input.run_id.to_string());
        state.runs.insert(input.run_id.to_string(), run);
        drop(state);
        Ok(PreparedIngestRun {
            run_record,
            stored_documents,
            run_to_store,
            run_file_hashes,
            diagnostics,
            document_events,
        })
    }
}

struct PersistPreparedIngestInput<'a> {
    request: &'a IngestStartRequest,
    run_id: &'a str,
    profile_id: &'a str,
    model_id: &'a str,
    file_count: usize,
    prepared: &'a PreparedIngestRun,
}

struct IngestPrepareInput<'a> {
    request: &'a IngestStartRequest,
    root: &'a Path,
    files: &'a [DiscoveredFile],
    run_id: &'a str,
    profile_id: &'a str,
    model_id: &'a str,
    runtime_id: &'a str,
}

struct PreparedIngestRun {
    run_record: IngestRunRecord,
    stored_documents: Vec<StoredDocument>,
    run_to_store: StoredRun,
    run_file_hashes: Vec<String>,
    diagnostics: Vec<(String, Value)>,
    document_events: Vec<DocumentSummary>,
}

fn checked_file_count(count: usize) -> Result<u32> {
    u32::try_from(count)
        .map_err(|_| AppError::BadRequest("too many supported files to ingest".to_string()))
}
