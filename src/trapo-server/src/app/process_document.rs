impl AppState {
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
        let render_work_unit_id = self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
            run_id,
            file_hash,
            page_no: None,
            phase: "render",
            model: ocr_context.model_id,
            profile: ocr_context.profile_id,
            metadata: json!({ "source_path": document_path.to_string_lossy().to_string() }),
        });
        self.start_diagnostic_work_unit(run_id, &render_work_unit_id);
        let render_span = DiagnosticSpanScope::start();
        let render_result = if is_pdf(&document_path) {
            self.log_info(
                "pdfium",
                format!(
                    "rendering {} at {PDF_DPI} DPI with PDFium",
                    document_path.display()
                ),
            )
            .await;
            self.inner.renderer.render_pdf(file_hash, &document_path)
        } else {
            self.inner
                .renderer
                .image_page(&document_path)
                .map(|page| vec![page])
        };
        let rendered = match render_result {
            Ok(rendered) => {
                self.record_span(
                    render_span,
                    SpanFinish {
                        run_id,
                        file_hash: Some(file_hash),
                        page_no: None,
                        name: "Render document",
                        pipeline_step: "render",
                        category: "file",
                        engine: Some("pdfium"),
                        status: "ok",
                        error: None,
                        attributes: json!({ "page_count": rendered.len() }),
                    },
                );
                self.finish_diagnostic_work_unit(
                    run_id,
                    &render_work_unit_id,
                    "completed",
                    None,
                    json!({ "page_count": rendered.len() }),
                );
                rendered
            }
            Err(error) => {
                let message = error.to_string();
                self.record_span(
                    render_span,
                    SpanFinish {
                        run_id,
                        file_hash: Some(file_hash),
                        page_no: None,
                        name: "Render document",
                        pipeline_step: "render",
                        category: "file",
                        engine: Some("pdfium"),
                        status: "failed",
                        error: Some(&message),
                        attributes: json!({}),
                    },
                );
                self.finish_diagnostic_work_unit(
                    run_id,
                    &render_work_unit_id,
                    "failed",
                    Some(&message),
                    json!({}),
                );
                return Err(error);
            }
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
                self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                    run_id,
                    file_hash,
                    page_no: Some(page.page_no),
                    phase: "ocr",
                    model: ocr_context.model_id,
                    profile: ocr_context.profile_id,
                    metadata: json!({
                        "width_px": page.width_px,
                        "height_px": page.height_px,
                        "render_dpi": page.render_dpi
                    }),
                });
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
            let page_work_unit_id =
                diagnostic_work_unit_id(run_id, file_hash, Some(page.page_no), "ocr");
            self.start_diagnostic_work_unit(run_id, &page_work_unit_id);
            let page_span = DiagnosticSpanScope::start();
            let page_result = self
                .process_page(
                    PageWork {
                        run_id,
                        file_hash,
                        image_path: &page.image_path,
                        page_no: page.page_no,
                    },
                    ocr_context,
                )
                .await;
            match page_result {
                Ok(()) => {
                    self.record_span(
                        page_span,
                        SpanFinish {
                            run_id,
                            file_hash: Some(file_hash),
                            page_no: Some(page.page_no),
                            name: "OCR page",
                            pipeline_step: "ocr",
                            category: "page",
                            engine: Some(ENGINE_ID),
                            status: "ok",
                            error: None,
                            attributes: json!({
                                "image_path": page.image_path.to_string_lossy().to_string()
                            }),
                        },
                    );
                    self.finish_diagnostic_work_unit(
                        run_id,
                        &page_work_unit_id,
                        "completed",
                        None,
                        json!({ "page_no": page.page_no }),
                    );
                }
                Err(error) => {
                    let message = error.to_string();
                    self.record_span(
                        page_span,
                        SpanFinish {
                            run_id,
                            file_hash: Some(file_hash),
                            page_no: Some(page.page_no),
                            name: "OCR page",
                            pipeline_step: "ocr",
                            category: "page",
                            engine: Some(ENGINE_ID),
                            status: "failed",
                            error: Some(&message),
                            attributes: json!({
                                "image_path": page.image_path.to_string_lossy().to_string()
                            }),
                        },
                    );
                    self.finish_diagnostic_work_unit(
                        run_id,
                        &page_work_unit_id,
                        "failed",
                        Some(&message),
                        json!({ "page_no": page.page_no }),
                    );
                    return Err(error);
                }
            }
        }
        self.finish_document(run_id, file_hash, "completed", None)
            .await
    }
}
