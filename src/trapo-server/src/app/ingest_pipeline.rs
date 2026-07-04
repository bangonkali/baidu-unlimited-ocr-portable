impl AppState {
    fn spawn_ingest(
        &self,
        run_id: String,
        files: Vec<DiscoveredFile>,
        profile_id: String,
        model_id: String,
        runtime_id: String,
    ) {
        let state = self.clone();
        tokio::spawn(async move {
            state
                .run_ingest(run_id, files, profile_id, model_id, runtime_id)
                .await;
        });
    }

    async fn run_ingest(
        &self,
        run_id: String,
        files: Vec<DiscoveredFile>,
        profile_id: String,
        model_id: String,
        runtime_id: String,
    ) {
        self.mark_run_status(&run_id, "running", None).await;
        let lease_scope = DiagnosticSpanScope::start();
        let ocr_worker = self
            .create_ocr_worker(&runtime_id, &profile_id, &model_id)
            .await;
        let fallback_reason = ocr_worker.fallback_error().map(ToString::to_string);
        self.record_model_lease(
            ModelLeaseDiagnostic {
                run_id: &run_id,
                model_id: &model_id,
                runtime_id: &runtime_id,
                profile_id: &profile_id,
                error: fallback_reason.as_deref(),
            },
            &lease_scope,
        );
        if let Some(reason) = fallback_reason.as_deref() {
            self.log_warn("ocr", format!("using fallback OCR text: {reason}"));
        }
        let (runtime_platform, accelerator) = self.runtime_stream_metadata(&runtime_id).await;
        let ocr_context = OcrRunContext {
            profile_id: &profile_id,
            model_id: &model_id,
            runtime_id: &runtime_id,
            runtime_platform: &runtime_platform,
            accelerator: &accelerator,
            worker: &ocr_worker,
        };
        for file in files {
            if self.run_cancelled(&run_id).await {
                break;
            }
            let file_hash = stable_hash(&file);
            if let Err(error) = self
                .process_document(&run_id, &file_hash, &ocr_context)
                .await
            {
                self.record_diagnostic_event(
                    &run_id,
                    Some(&file_hash),
                    None,
                    "error",
                    &error.to_string(),
                );
                self.mark_document_error(&run_id, &file_hash, error.to_string())
                    .await;
            }
        }
        let final_status = if self.run_cancelled(&run_id).await {
            "cancelled"
        } else if self.run_has_errors(&run_id).await {
            "completed_with_errors"
        } else {
            "completed"
        };
        self.mark_run_status(&run_id, final_status, None).await;
        {
            let mut state = self.inner.state.lock().await;
            if state.active_run_id.as_deref() == Some(&run_id) {
                state.active_run_id = None;
            }
        }
        self.publish_status_changed().await;
    }
}
