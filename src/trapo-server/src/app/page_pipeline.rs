impl AppState {
    async fn process_page(
        &self,
        run_id: &str,
        file_hash: &str,
        image_path: &Path,
        page_no: u32,
        profile_id: &str,
        model_id: &str,
    ) -> Result<()> {
        let started = Instant::now();
        let raw_text = self
            .run_ocr_or_fallback(image_path, file_hash, page_no, profile_id, model_id)
            .await;
        let mut parsed = crate::ocr::parse_ocr_markers(
            &raw_text,
            &crate::ocr::ParseContext {
                file_hash: file_hash.to_string(),
                page_no,
                engine_id: ENGINE_ID.to_string(),
                profile_id: profile_id.to_string(),
            },
        );
        crate::ocr::apply_region_content(&mut parsed);
        let (page_record, regions_payload, text_payload) = {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            let (stored, width_px, height_px) = {
                let page = document
                    .pages
                    .iter_mut()
                    .find(|item| item.page_no == page_no)
                    .ok_or_else(|| AppError::NotFound("page not found".to_string()))?;
                page.status = "completed".to_string();
                page.raw_text = parsed.raw_text;
                page.cleaned_text = if parsed.cleaned_text.is_empty() {
                    raw_text
                } else {
                    parsed.cleaned_text
                };
                page.boxes = parsed.boxes;
                page.spans = parsed.spans;
                (stored_page(file_hash, page), page.width_px, page.height_px)
            };
            self.inner.repository.replace_page_ocr(
                &stored,
                ENGINE_ID,
                profile_id,
                started.elapsed().as_millis() as u64,
            )?;
            let regions = DocumentRegionsPayload {
                file_hash: file_hash.to_string(),
                boxes: document
                    .pages
                    .iter()
                    .flat_map(|page| page.boxes.clone())
                    .collect(),
            };
            let text = DocumentTextPayload {
                file_hash: file_hash.to_string(),
                pages: document
                    .pages
                    .iter()
                    .map(|page| PageTextRecord {
                        page_no: page.page_no,
                        text: page.cleaned_text.clone(),
                        spans: page.spans.clone(),
                    })
                    .collect(),
            };
            let page_record = json!({
                "file_hash": file_hash,
                "page_no": page_no,
                "status": "completed",
                "width_px": width_px,
                "height_px": height_px,
            });
            (page_record, regions, text)
        };
        self.increment_run_page(run_id, file_hash).await?;
        self.inner.hub.publish("document.page.changed", page_record);
        self.inner.hub.publish(
            "document.regions.changed",
            serde_json::to_value(regions_payload)?,
        );
        self.inner
            .hub
            .publish("document.text.changed", serde_json::to_value(text_payload)?);
        self.inner.repository.upsert_page_metrics(&OcrPageMetrics {
            run_id: run_id.to_string(),
            file_hash: file_hash.to_string(),
            page_no,
            model_id: model_id.to_string(),
            runtime_id: self.selected_runtime_id().await,
            status: "completed".to_string(),
            token_count: 0,
            avg_tps: 0.0,
            elapsed_ms: started.elapsed().as_millis() as u64,
        })?;
        Ok(())
    }

    async fn run_ocr_or_fallback(
        &self,
        image_path: &Path,
        file_hash: &str,
        page_no: u32,
        profile_id: &str,
        model_id: &str,
    ) -> String {
        let (runtime, profile, model_file) = {
            let state = self.inner.state.lock().await;
            (
                selected_runtime(&state).cloned(),
                find_profile(profile_id),
                find_model(model_id).map(|entry| entry.model_file),
            )
        };
        let Some(runtime) = runtime.filter(|item| item.selectable) else {
            return fallback_text(image_path, "runtime is not selectable");
        };
        let Some(profile) = profile else {
            return fallback_text(image_path, "OCR profile was not found");
        };
        let Some(model_file) = model_file else {
            return fallback_text(image_path, "model was not found");
        };
        let paths = crate::ocr::runtime_paths(&self.inner.config.app_root, &runtime, model_file);
        if !paths.model.is_file() || !paths.mmproj.is_file() || !paths.ffi_library.is_file() {
            return fallback_text(image_path, "native OCR assets are not installed");
        }
        let mut engine = match crate::ocr::UnlimitedOcrFfiEngine::load(paths, &profile) {
            Ok(engine) => engine,
            Err(error) => return fallback_text(image_path, &error.to_string()),
        };
        self.inner.hub.publish(
            "ocr.page.stream.started",
            json!({ "file_hash": file_hash, "page_no": page_no, "profile_id": profile_id, "model_id": model_id }),
        );
        let result = engine.recognize_image(image_path, profile.default_max_tokens as i32, |event| {
            if let crate::ocr::OcrEvent::Token { text, index } = event {
                self.inner.hub.publish(
                    "ocr.page.raw.delta",
                    json!({ "file_hash": file_hash, "page_no": page_no, "text": text, "index": index }),
                );
            }
        });
        if result.ok {
            self.inner.hub.publish(
                "ocr.page.stream.completed",
                json!({ "file_hash": file_hash, "page_no": page_no }),
            );
            result.text
        } else {
            let message = result
                .error
                .unwrap_or_else(|| "uocr-ffi failed".to_string());
            self.inner.hub.publish(
                "ocr.page.stream.failed",
                json!({ "file_hash": file_hash, "page_no": page_no, "error": message }),
            );
            fallback_text(image_path, &message)
        }
    }

    async fn increment_run_page(&self, run_id: &str, file_hash: &str) -> Result<()> {
        let (run_event, document_event) = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return Err(AppError::NotFound("run not found".to_string()));
            };
            run.processed_pages = run.processed_pages.saturating_add(1);
            run.current_page = Some(run.processed_pages);
            self.inner.repository.upsert_run(&stored_run(run))?;
            let run_event = run_record(run);
            let document_event = state.documents.get(file_hash).map(document_summary);
            (run_event, document_event)
        };
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
        let event = {
            let mut state = self.inner.state.lock().await;
            let Some(document) = state.documents.get_mut(file_hash) else {
                return Err(AppError::NotFound("document not found".to_string()));
            };
            document.status = status.to_string();
            document.error = error;
            self.inner
                .repository
                .upsert_document(&stored_document(document))?;
            document_summary(document)
        };
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
        self.log_error("ingest", error).await;
    }

    async fn mark_run_status(&self, run_id: &str, status: &str, error: Option<String>) {
        let event = {
            let mut state = self.inner.state.lock().await;
            let Some(run) = state.runs.get_mut(run_id) else {
                return;
            };
            run.status = status.to_string();
            if error.is_some() {
                run.error = error;
            }
            let _ = self.inner.repository.upsert_run(&stored_run(run));
            run_record(run)
        };
        self.inner.hub.publish(
            "run.changed",
            serde_json::to_value(event).unwrap_or_else(|_| json!({})),
        );
    }

    async fn run_cancelled(&self, run_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        state
            .runs
            .get(run_id)
            .map(|run| run.cancel_requested || run.status == "cancelled")
            .unwrap_or(true)
    }

    async fn run_has_errors(&self, run_id: &str) -> bool {
        let state = self.inner.state.lock().await;
        state
            .runs
            .get(run_id)
            .map(|run| {
                run.file_hashes.iter().any(|hash| {
                    state
                        .documents
                        .get(hash)
                        .is_some_and(|document| document.status == "failed")
                })
            })
            .unwrap_or(false)
    }

    async fn selected_runtime_id(&self) -> String {
        let state = self.inner.state.lock().await;
        state.selected_runtime_id.clone()
    }
}
