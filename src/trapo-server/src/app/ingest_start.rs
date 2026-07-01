impl AppState {
    pub async fn start_ingest(&self, request: IngestStartRequest) -> Result<IngestRunRecord> {
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
        let run_record = {
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
            for file in &files {
                let document = document_from_file(file, &root);
                run.file_hashes.push(document.file_hash.clone());
                self.inner
                    .repository
                    .upsert_document(&stored_document(&document))?;
                document_events.push(document_summary(&document));
                state.documents.insert(document.file_hash.clone(), document);
            }
            self.inner.repository.upsert_run(&stored_run(&run))?;
            let record = run_record(&run);
            state.active_run_id = Some(run_id.clone());
            state.runs.insert(run_id.clone(), run);
            record
        };
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
        for event in document_events {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(event)?);
        }
        self.publish_status_changed().await;
        self.spawn_ingest(run_id.clone(), files, profile_id, model_id);
        self.get_run(&run_id).await
    }
}
