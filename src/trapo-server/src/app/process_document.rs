impl AppState {
    async fn process_document(
        &self,
        run_id: &str,
        file_hash: &str,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<()> {
        let document_path = self.mark_document_rendering(file_hash).await?;
        let rendered = self.render_document_pages(run_id, file_hash, &document_path, ocr_context)?;
        self.queue_rendered_pages(run_id, file_hash, &rendered, ocr_context)
            .await?;
        self.process_rendered_pages(run_id, file_hash, rendered, ocr_context)
            .await?;
        self.finish_document(run_id, file_hash, "completed", None)
            .await
    }

    async fn mark_document_rendering(&self, file_hash: &str) -> Result<PathBuf> {
        let (document_path, stored, event) = {
            let mut state = self.inner.state.lock().await;
            let document = state
                .documents
                .get_mut(file_hash)
                .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
            document.status = "rendering".to_string();
            let path = document.absolute_path.clone();
            let stored = stored_document(document);
            let event = document_summary(document);
            drop(state);
            (path, stored, event)
        };
        self.inner.repository.upsert_document(&stored).await?;
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(event)?);
        Ok(document_path)
    }

    fn render_document_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        document_path: &Path,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<Vec<RenderedPage>> {
        let work_unit_id = self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
            run_id,
            file_hash,
            page_no: None,
            phase: "render",
            model: ocr_context.model_id,
            profile: ocr_context.profile_id,
            metadata: json!({ "source_path": document_path.to_string_lossy().to_string() }),
        });
        self.start_diagnostic_work_unit(run_id, &work_unit_id);
        let span = DiagnosticSpanScope::start();
        match self.render_document_file(file_hash, document_path) {
            Ok(rendered) => {
                self.finish_render_diagnostics(run_id, file_hash, &work_unit_id, span, &rendered);
                Ok(rendered)
            }
            Err(error) => {
                self.fail_render_diagnostics(run_id, file_hash, &work_unit_id, span, &error);
                Err(error)
            }
        }
    }

    fn render_document_file(
        &self,
        file_hash: &str,
        document_path: &Path,
    ) -> Result<Vec<RenderedPage>> {
        if is_pdf(document_path) {
            self.log_info(
                "pdfium",
                format!(
                    "rendering {} at {PDF_DPI} DPI with PDFium",
                    document_path.display()
                ),
            );
            self.inner.renderer.render_pdf(file_hash, document_path)
        } else {
            PdfRenderer::image_page(document_path).map(|page| vec![page])
        }
    }

    fn finish_render_diagnostics(
        &self,
        run_id: &str,
        file_hash: &str,
        work_unit_id: &str,
        span: DiagnosticSpanScope,
        rendered: &[RenderedPage],
    ) {
        self.record_span(
            span,
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
            work_unit_id,
            "completed",
            None,
            json!({ "page_count": rendered.len() }),
        );
    }

    fn fail_render_diagnostics(
        &self,
        run_id: &str,
        file_hash: &str,
        work_unit_id: &str,
        span: DiagnosticSpanScope,
        error: &AppError,
    ) {
        let message = error.to_string();
        self.record_span(
            span,
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
        self.finish_diagnostic_work_unit(run_id, work_unit_id, "failed", Some(&message), json!({}));
    }

    async fn queue_rendered_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        rendered: &[RenderedPage],
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<()> {
        let (stored, pages, event, page_diagnostics) =
            self.update_document_pages(file_hash, rendered).await?;
        self.inner.repository.upsert_document(&stored).await?;
        for page in &pages {
            self.inner.repository.upsert_page(page).await?;
        }
        for (page_no, metadata) in page_diagnostics {
            self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                run_id,
                file_hash,
                page_no: Some(page_no),
                phase: "ocr",
                model: ocr_context.model_id,
                profile: ocr_context.profile_id,
                metadata,
            });
        }
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(event)?);
        Ok(())
    }

    async fn update_document_pages(
        &self,
        file_hash: &str,
        rendered: &[RenderedPage],
    ) -> Result<(StoredDocument, Vec<StoredPage>, DocumentSummary, Vec<(u32, Value)>)> {
        let mut state = self.inner.state.lock().await;
        let document = state
            .documents
            .get_mut(file_hash)
            .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
        document.page_count = usize_to_u32_saturating(rendered.len());
        document.status = "running".to_string();
        document.pages = rendered.iter().map(queued_page_state).collect();
        let pages = document
            .pages
            .iter()
            .map(|page| stored_page(file_hash, page))
            .collect();
        let diagnostics = document
            .pages
            .iter()
            .map(page_diagnostic_metadata)
            .collect();
        let stored = stored_document(document);
        let event = document_summary(document);
        drop(state);
        Ok((stored, pages, event, diagnostics))
    }

    async fn process_rendered_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        rendered: Vec<RenderedPage>,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<()> {
        for page in rendered {
            if self.run_cancelled(run_id).await {
                return Ok(());
            }
            self.process_rendered_page(run_id, file_hash, &page, ocr_context)
                .await?;
        }
        Ok(())
    }

    async fn process_rendered_page(
        &self,
        run_id: &str,
        file_hash: &str,
        page: &RenderedPage,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<()> {
        let work_unit_id = diagnostic_work_unit_id(run_id, file_hash, Some(page.page_no), "ocr");
        self.start_diagnostic_work_unit(run_id, &work_unit_id);
        let span = DiagnosticSpanScope::start();
        let result = self
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
        self.finish_page_diagnostics(
            PageDiagnosticFinish {
                run_id,
                file_hash,
                page,
                work_unit_id: &work_unit_id,
                result: &result,
            },
            span,
        );
        result
    }

    fn finish_page_diagnostics(&self, finish: PageDiagnosticFinish<'_>, span: DiagnosticSpanScope) {
        let (status, error) = match finish.result {
            Ok(()) => ("ok", None),
            Err(error) => ("failed", Some(error.to_string())),
        };
        let error_ref = error.as_deref();
        self.record_span(
            span,
            SpanFinish {
                run_id: finish.run_id,
                file_hash: Some(finish.file_hash),
                page_no: Some(finish.page.page_no),
                name: "OCR page",
                pipeline_step: "ocr",
                category: "page",
                engine: Some(ENGINE_ID),
                status,
                error: error_ref,
                attributes: json!({
                    "image_path": finish.page.image_path.to_string_lossy().to_string()
                }),
            },
        );
        self.finish_diagnostic_work_unit(
            finish.run_id,
            finish.work_unit_id,
            if finish.result.is_ok() {
                "completed"
            } else {
                "failed"
            },
            error_ref,
            json!({ "page_no": finish.page.page_no }),
        );
    }
}
