impl AppState {
    pub async fn start_ingest(&self, request: IngestStartRequest) -> Result<IngestStartResponse> {
        let root = PathBuf::from(&request.root_path);
        let files = discover_supported_files(&root)?;
        let run_id = now_id();
        let (profile_id, model_id, runtime_id) = {
            let state = self.inner.state.lock().await;
            (
                request
                    .profile_id
                    .clone()
                    .unwrap_or_else(|| state.selected_profile_id.clone()),
                request
                    .model_id
                    .clone()
                    .unwrap_or_else(|| state.selected_model_id.clone()),
                state.selected_runtime_id.clone(),
            )
        };
        if find_profile(&profile_id).is_none() {
            return Err(AppError::BadRequest(format!(
                "unknown OCR profile: {profile_id}"
            )));
        }
        if find_model(&model_id).is_none() {
            return Err(AppError::BadRequest(format!(
                "unknown model id: {model_id}"
            )));
        }

        let mut document_events = Vec::new();
        let (run_record, stored_documents, run_to_store, run_file_hashes, diagnostics) = {
            let mut state = self.inner.state.lock().await;
            let mut run = RunState {
                run_id: run_id.clone(),
                root_path: request.root_path.clone(),
                status: if files.is_empty() {
                    "completed".to_string()
                } else {
                    "queued".to_string()
                },
                queued_files: files.len() as u32,
                processed_pages: 0,
                total_pages: files.len() as u32,
                current_page: None,
                profile_id: profile_id.clone(),
                engine_id: request.engine_id.unwrap_or_else(|| ENGINE_ID.to_string()),
                model_id: model_id.clone(),
                runtime_id: runtime_id.clone(),
                error: None,
                cancel_requested: false,
                file_hashes: Vec::new(),
            };
            let mut stored_documents = Vec::new();
            let mut diagnostics = Vec::new();
            for file in &files {
                let document = document_from_file(file, &root); // skylos: ignore[SKY-D215] files come from discover_supported_files under validate_trusted_root().
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
            let record = run_record(&run);
            let run_to_store = stored_run(&run);
            let run_file_hashes = run.file_hashes.clone();
            state.active_run_id = Some(run_id.clone());
            state.runs.insert(run_id.clone(), run);
            (
                record,
                stored_documents,
                run_to_store,
                run_file_hashes,
                diagnostics,
            )
        };
        for document in &stored_documents {
            self.inner.repository.upsert_document(document).await?;
        }
        for (file_hash, metadata) in diagnostics {
            self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                run_id: &run_id,
                file_hash: &file_hash,
                page_no: None,
                phase: "render",
                model: &model_id,
                profile: &profile_id,
                metadata,
            });
        }
        self.inner.repository.upsert_run(&run_to_store).await?;
        self.inner
            .repository
            .replace_run_documents(&run_id, &run_file_hashes)
            .await?;
        self.log_info(
            "ingest",
            format!(
                "scan requested for {} found {} supported files",
                request.root_path,
                files.len()
            ),
        )
        .await;
        self.inner
            .hub
            .publish("run.changed", serde_json::to_value(&run_record)?);
        for event in &document_events {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(event)?);
        }
        self.publish_status_changed().await;
        let replay_since_sequence = self.inner.hub.last_sequence();
        self.spawn_ingest(run_id.clone(), files, profile_id, model_id, runtime_id);
        Ok(IngestStartResponse {
            run: self.get_run(&run_id).await?,
            documents: document_events,
            replay_since_sequence,
        })
    }
}
