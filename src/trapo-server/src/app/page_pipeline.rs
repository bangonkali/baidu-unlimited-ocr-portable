impl AppState {
    async fn process_page(&self, page_work: PageWork<'_>, ocr: &OcrRunContext<'_>) -> Result<()> {
        let started = Instant::now();
        self.mark_page_started(page_work.file_hash, page_work.page_no)
            .await?;
        let parsed = self.parse_page_output(&page_work, ocr)?;
        let completed = self.complete_page_state(&page_work, ocr, parsed).await?;
        self.persist_completed_page(&page_work, ocr, started, completed)
            .await?;
        Ok(())
    }

    async fn mark_page_started(&self, file_hash: &str, page_no: u32) -> Result<()> {
        let (stored, page_record, document_event) = {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            let page = document
                .pages
                .iter_mut()
                .find(|item| item.page_no == page_no)
                .ok_or_else(|| AppError::NotFound("page not found".to_string()))?;
            page.status = "running".to_string();
            let stored = stored_page(file_hash, page);
            let page_record = json!({
                "file_hash": file_hash,
                "page_no": page_no,
                "status": "running",
                "width_px": page.width_px,
                "height_px": page.height_px,
                "dpi": page.render_dpi,
                "preview_available": true,
            });
            let document_event = document_summary(document);
            drop(state);
            (stored, page_record, document_event)
        };
        self.inner.repository.upsert_page(&stored).await?;
        self.inner.hub.publish("document.page.changed", page_record);
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(document_event)?);
        Ok(())
    }

    fn run_ocr_or_fallback(
        &self,
        image_path: &Path,
        context: &OcrStreamContext,
        ocr_worker: &OcrRunWorker,
    ) -> Result<String> {
        let mut started = stream_context_payload(context);
        started["started_at"] = json!(Utc::now().to_rfc3339());
        self.inner.hub.publish("ocr.page.stream.started", started);
        let result = ocr_worker.recognize(image_path, context.clone());
        if result.ok {
            self.inner.hub.publish(
                "ocr.page.stream.completed",
                stream_terminal_payload(context, "completed", None),
            );
            Ok(result.text)
        } else {
            let message = result
                .error
                .unwrap_or_else(|| "uocr-ffi failed".to_string());
            self.inner.hub.publish(
                "ocr.page.stream.failed",
                stream_terminal_payload(context, "failed", Some(&message)),
            );
            Err(AppError::Internal(message))
        }
    }

    async fn increment_run_page(&self, run_id: &str, file_hash: &str) -> Result<()> {
        let (stored, run_event, document_event) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return Err(AppError::NotFound("run not found".to_string()));
            };
            run.processed_pages = run.processed_pages.saturating_add(1);
            run.current_page = Some(run.processed_pages);
            let stored = stored_run(run);
            let run_event = run_record(run);
            let document_event = state.documents.get(file_hash).map(document_summary);
            drop(state);
            (stored, run_event, document_event)
        };
        self.inner.repository.upsert_run(&stored).await?;
        self.inner
            .hub
            .publish("run.changed", serde_json::to_value(run_event)?);
        if let Some(document_event) = document_event {
            self.inner
                .hub
                .publish("document.changed", serde_json::to_value(document_event)?);
        }
        Ok(())
    }

    async fn finish_document(
        &self,
        run_id: &str,
        file_hash: &str,
        status: &str,
        error: Option<String>,
    ) -> Result<()> {
        let (stored, event) = {
            let mut state = self.inner.state.lock().await;
            let Some(document) = state.documents.get_mut(file_hash) else {
                return Err(AppError::NotFound("document not found".to_string()));
            };
            document.status = status.to_string();
            document.error = error;
            let stored = stored_document(document);
            let event = document_summary(document);
            drop(state);
            (stored, event)
        };
        self.inner.repository.upsert_document(&stored).await?;
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(event)?);
        let _ = run_id;
        Ok(())
    }

    async fn mark_document_error(&self, run_id: &str, file_hash: &str, error: String) {
        let _ = self
            .finish_document(run_id, file_hash, "failed", Some(error.clone()))
            .await;
        self.log_error("ingest", error);
    }

    async fn mark_run_status(&self, run_id: &str, status: &str, error: Option<String>) {
        let (stored, event) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return;
            };
            run.status = status.to_string();
            if error.is_some() {
                run.error = error;
            }
            let stored = stored_run(run);
            let event = run_record(run);
            drop(state);
            (stored, event)
        };
        if let Err(error) = self.inner.repository.upsert_run(&stored).await {
            tracing::warn!(%error, run_id, "failed to persist run status");
        }
        self.inner.hub.publish(
            "run.changed",
            serde_json::to_value(event).unwrap_or_else(|_| json!({})),
        );
    }

    async fn mark_engine_status(
        &self,
        run_engine_id: &str,
        status: &str,
        error: Option<&str>,
        usable_output_count: u32,
    ) {
        let (stored, event) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.values_mut().find(|run| {
                run.engine_configs
                    .iter()
                    .any(|config| config.run_engine_id == run_engine_id)
            }) else {
                return;
            };
            let stored = {
                let Some(config) = run
                    .engine_configs
                    .iter_mut()
                    .find(|config| config.run_engine_id == run_engine_id)
                else {
                    return;
                };
                config.status = status.to_string();
                config.error = error.map(ToString::to_string);
                config.usable_output_count = usable_output_count;
                stored_run_engine_config(config)
            };
            let event = run_record(run);
            drop(state);
            (stored, event)
        };
        if let Err(error) = self
            .inner
            .repository
            .update_run_engine_config_status(
                run_engine_id,
                status,
                error,
                stored.usable_output_count,
            )
            .await
        {
            tracing::warn!(%error, run_engine_id, "failed to persist engine status");
        }
        self.inner.hub.publish(
            "run.changed",
            serde_json::to_value(event).unwrap_or_else(|_| json!({})),
        );
    }

    async fn run_cancelled(&self, run_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        let cancelled = state
            .runs
            .get(run_id)
            .is_none_or(|run| run.cancel_requested || run.status == "cancelled");
        drop(state);
        cancelled
    }

    async fn run_has_errors(&self, run_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        let has_errors = state
            .runs
            .get(run_id)
            .is_some_and(|run| {
                run.file_hashes.iter().any(|hash| {
                    state
                        .documents
                        .get(hash)
                        .is_some_and(|document| document.status == "failed")
                })
            });
        drop(state);
        has_errors
    }
}

impl PageWork<'_> {
    fn stream_context(&self, ocr: &OcrRunContext<'_>) -> OcrStreamContext {
        OcrStreamContext {
            run_id: self.run_id.to_string(),
            run_engine_id: ocr.run_engine_id.to_string(),
            file_hash: self.file_hash.to_string(),
            page_no: self.page_no,
            engine_id: ocr.engine_id.to_string(),
            profile_id: ocr.profile_id.to_string(),
            model_id: ocr.model_id.to_string(),
            runtime_id: ocr.runtime_id.to_string(),
            runtime_platform: ocr.runtime_platform.to_string(),
            accelerator: ocr.accelerator.to_string(),
        }
    }
}
