impl AppState {
    async fn process_page(&self, page_work: PageWork<'_>, ocr: &OcrRunContext<'_>) -> Result<()> {
        let started = Instant::now();
        self.mark_page_started(page_work.file_hash, page_work.page_no)
            .await?;
        let raw_text = self
            .run_ocr_or_fallback(
                page_work.image_path,
                &page_work.stream_context(ocr),
                ocr.worker,
            )
            .await;
        let mut parsed = crate::ocr::parse_ocr_markers(
            &raw_text,
            &crate::ocr::ParseContext {
                file_hash: page_work.file_hash.to_string(),
                page_no: page_work.page_no,
                engine_id: ENGINE_ID.to_string(),
                profile_id: ocr.profile_id.to_string(),
            },
        );
        crate::ocr::apply_region_content(&mut parsed);
        self.write_image_region_snippets(
            page_work.file_hash,
            page_work.image_path,
            &mut parsed.boxes,
        )?;
        let (page_record, regions_payload, text_payload) = {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(page_work.file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            let (stored, width_px, height_px) = {
                let page = document
                    .pages
                    .iter_mut()
                    .find(|item| item.page_no == page_work.page_no)
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
                (
                    stored_page(page_work.file_hash, page),
                    page.width_px,
                    page.height_px,
                )
            };
            self.inner.repository.replace_page_ocr(
                &stored,
                ENGINE_ID,
                ocr.profile_id,
                started.elapsed().as_millis() as u64,
            )?;
            let regions = DocumentRegionsPayload {
                file_hash: page_work.file_hash.to_string(),
                boxes: document
                    .pages
                    .iter()
                    .flat_map(|page| page.boxes.clone())
                    .collect(),
            };
            let text = DocumentTextPayload {
                file_hash: page_work.file_hash.to_string(),
                pages: started_page_text_records(document),
            };
            let page_record = json!({
                "file_hash": page_work.file_hash,
                "page_no": page_work.page_no,
                "status": "completed",
                "width_px": width_px,
                "height_px": height_px,
            });
            (page_record, regions, text)
        };
        self.increment_run_page(page_work.run_id, page_work.file_hash)
            .await?;
        self.inner.hub.publish("document.page.changed", page_record);
        self.inner.hub.publish(
            "document.regions.changed",
            serde_json::to_value(regions_payload)?,
        );
        self.inner
            .hub
            .publish("document.text.changed", serde_json::to_value(text_payload)?);
        self.inner.repository.upsert_page_metrics(&OcrPageMetrics {
            run_id: page_work.run_id.to_string(),
            file_hash: page_work.file_hash.to_string(),
            page_no: page_work.page_no,
            model_id: ocr.model_id.to_string(),
            runtime_id: ocr.runtime_id.to_string(),
            status: "completed".to_string(),
            token_count: 0,
            avg_tps: 0.0,
            elapsed_ms: started.elapsed().as_millis() as u64,
        })?;
        Ok(())
    }

    async fn mark_page_started(&self, file_hash: &str, page_no: u32) -> Result<()> {
        let (page_record, document_event) = {
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
            self.inner
                .repository
                .upsert_page(&stored_page(file_hash, page))?;
            let page_record = json!({
                "file_hash": file_hash,
                "page_no": page_no,
                "status": "running",
                "width_px": page.width_px,
                "height_px": page.height_px,
            });
            (page_record, document_summary(document))
        };
        self.inner.hub.publish("document.page.changed", page_record);
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(document_event)?);
        Ok(())
    }

    async fn run_ocr_or_fallback(
        &self,
        image_path: &Path,
        context: &OcrStreamContext,
        ocr_worker: &OcrRunWorker,
    ) -> String {
        let mut started = stream_context_payload(context);
        started["started_at"] = json!(Utc::now().to_rfc3339());
        self.inner.hub.publish("ocr.page.stream.started", started);
        let result = ocr_worker.recognize(image_path, context.clone());
        if result.ok {
            self.inner.hub.publish(
                "ocr.page.stream.completed",
                stream_terminal_payload(context, "completed", None),
            );
            result.text
        } else {
            let message = result
                .error
                .unwrap_or_else(|| "uocr-ffi failed".to_string());
            self.inner.hub.publish(
                "ocr.page.stream.failed",
                stream_terminal_payload(context, "failed", Some(&message)),
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

}

impl<'a> PageWork<'a> {
    fn stream_context(&self, ocr: &OcrRunContext<'_>) -> OcrStreamContext {
        OcrStreamContext {
            run_id: self.run_id.to_string(),
            file_hash: self.file_hash.to_string(),
            page_no: self.page_no,
            engine_id: ENGINE_ID.to_string(),
            profile_id: ocr.profile_id.to_string(),
            model_id: ocr.model_id.to_string(),
            runtime_id: ocr.runtime_id.to_string(),
            runtime_platform: ocr.runtime_platform.to_string(),
            accelerator: ocr.accelerator.to_string(),
        }
    }
}
