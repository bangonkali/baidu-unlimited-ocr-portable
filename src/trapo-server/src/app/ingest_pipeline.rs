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
        let ocr_worker = self
            .create_ocr_worker(&runtime_id, &profile_id, &model_id)
            .await;
        if let Some(reason) = ocr_worker.fallback_error() {
            self.log_warn("ocr", format!("using fallback OCR text: {reason}"))
                .await;
        }
        let ocr_context = OcrRunContext {
            profile_id: &profile_id,
            model_id: &model_id,
            runtime_id: &runtime_id,
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

    async fn process_document(
        &self,
        run_id: &str,
        file_hash: &str,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<()> {
        let document_path = {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            document.status = "rendering".to_string();
            self.inner
                .repository
                .upsert_document(&stored_document(document))?;
            self.inner.hub.publish(
                "document.changed",
                serde_json::to_value(document_summary(document))?,
            );
            document.absolute_path.clone()
        };
        let rendered = if is_pdf(&document_path) {
            self.log_info(
                "pdfium",
                format!(
                    "rendering {} at {PDF_DPI} DPI with PDFium",
                    document_path.display()
                ),
            )
            .await;
            self.inner.renderer.render_pdf(file_hash, &document_path)?
        } else {
            vec![self.inner.renderer.image_page(&document_path)?]
        };
        {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            document.page_count = rendered.len() as u32;
            document.status = "running".to_string();
            document.pages = rendered
                .iter()
                .map(|page| PageState {
                    page_no: page.page_no,
                    image_path: page.image_path.clone(),
                    width_px: page.width_px,
                    height_px: page.height_px,
                    render_dpi: PDF_DPI,
                    status: "queued".to_string(),
                    raw_text: String::new(),
                    cleaned_text: String::new(),
                    boxes: Vec::new(),
                    spans: Vec::new(),
                    error: None,
                })
                .collect();
            self.inner
                .repository
                .upsert_document(&stored_document(document))?;
            for page in &document.pages {
                self.inner
                    .repository
                    .upsert_page(&stored_page(file_hash, page))?;
            }
            self.inner.hub.publish(
                "document.changed",
                serde_json::to_value(document_summary(document))?,
            );
        }
        for page in rendered {
            if self.run_cancelled(run_id).await {
                return Ok(());
            }
            self.process_page(
                PageWork {
                    run_id,
                    file_hash,
                    image_path: &page.image_path,
                    page_no: page.page_no,
                },
                ocr_context,
            )
            .await?;
        }
        self.finish_document(run_id, file_hash, "completed", None)
            .await
    }

}
