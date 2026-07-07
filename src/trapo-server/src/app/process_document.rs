impl AppState {
    async fn process_document(
        &self,
        run_id: &str,
        file_hash: &str,
        ocr_context: &OcrRunContext<'_>,
        completed_pages: &BTreeSet<(String, u32)>,
    ) -> Result<()> {
        let document_path = self.mark_document_rendering(file_hash).await?;
        let rendered = self
            .render_document_pages(run_id, file_hash, &document_path, ocr_context)
            .await?;
        self.queue_rendered_pages(run_id, file_hash, &rendered, ocr_context, completed_pages)
            .await?;
        self.process_rendered_pages(run_id, file_hash, rendered, ocr_context, completed_pages)
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

    async fn render_document_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        document_path: &Path,
        ocr_context: &OcrRunContext<'_>,
    ) -> Result<Vec<RenderedPage>> {
        let work_unit_id = self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
            run_id,
            run_engine_id: None,
            file_hash,
            page_no: None,
            phase: "render",
            engine: "pdfium",
            model: ocr_context.model_id,
            profile: Some(ocr_context.profile_id),
            metadata: json!({ "source_path": document_path.to_string_lossy().to_string() }),
        })
        .await;
        self.start_diagnostic_work_unit(run_id, &work_unit_id).await;
        let span = DiagnosticSpanScope::start();
        match self.render_document_file(file_hash, document_path) {
            Ok(rendered) => {
                self.finish_render_diagnostics(run_id, file_hash, &work_unit_id, span, &rendered)
                    .await;
                Ok(rendered)
            }
            Err(error) => {
                self.fail_render_diagnostics(run_id, file_hash, &work_unit_id, span, &error)
                    .await;
                Err(error)
            }
        }
    }

    async fn queue_rendered_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        rendered: &[RenderedPage],
        ocr_context: &OcrRunContext<'_>,
        completed_pages: &BTreeSet<(String, u32)>,
    ) -> Result<()> {
        let (stored, pages, event, page_diagnostics) =
            self.update_document_pages(run_id, file_hash, rendered, completed_pages)
                .await?;
        self.inner.repository.upsert_document(&stored).await?;
        for page in &pages {
            self.inner.repository.upsert_page(page).await?;
        }
        for page in &pages {
            self.inner
                .hub
                .publish("document.page.changed", rendered_page_record(page));
        }
        for (page_no, metadata) in page_diagnostics {
            self.upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                run_id,
                run_engine_id: Some(ocr_context.run_engine_id),
                file_hash,
                page_no: Some(page_no),
                phase: "ocr",
                engine: ocr_context.engine_id,
                model: ocr_context.model_id,
                profile: Some(ocr_context.profile_id),
                metadata,
            })
            .await;
        }
        self.inner
            .hub
            .publish("document.changed", serde_json::to_value(event)?);
        Ok(())
    }

    async fn update_document_pages(
        &self,
        run_id: &str,
        file_hash: &str,
        rendered: &[RenderedPage],
        completed_pages: &BTreeSet<(String, u32)>,
    ) -> Result<(StoredDocument, Vec<StoredPage>, DocumentSummary, Vec<(u32, Value)>)> {
        let mut state = self.inner.state.lock().await;
        let document = state
            .documents
            .get_mut(file_hash)
            .ok_or_else(|| AppError::NotFound("document not found".to_string()))?;
        document.page_count = usize_to_u32_saturating(rendered.len());
        document.status = "running".to_string();
        let existing_pages = document.pages.clone();
        document.pages = rendered
            .iter()
            .map(|page| {
                page_state_for_resume_queue(
                    page,
                    &existing_pages,
                    completed_pages.contains(&(file_hash.to_string(), page.page_no)),
                )
            })
            .collect();
        let pages = document
            .pages
            .iter()
            .map(|page| {
                let mut stored = stored_page(file_hash, page);
                stored.run_id = Some(run_id.to_string());
                stored
            })
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
        completed_pages: &BTreeSet<(String, u32)>,
    ) -> Result<()> {
        for page in rendered {
            if completed_pages.contains(&(file_hash.to_string(), page.page_no)) {
                continue;
            }
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
        let work_unit_id = self
            .upsert_diagnostic_work_unit(DiagnosticWorkUnitDraft {
                run_id,
                run_engine_id: Some(ocr_context.run_engine_id),
                file_hash,
                page_no: Some(page.page_no),
                phase: "ocr",
                engine: ocr_context.engine_id,
                model: ocr_context.model_id,
                profile: Some(ocr_context.profile_id),
                metadata: json!({ "image_path": page.image_path.to_string_lossy().to_string() }),
            })
            .await;
        self.start_diagnostic_work_unit(run_id, &work_unit_id).await;
        let span = DiagnosticSpanScope::start();
        let result = self
            .process_page(
                PageWork {
                    run_id,
                    work_unit_id: &work_unit_id.id,
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
                work_unit: &work_unit_id,
                engine_id: ocr_context.engine_id,
                result: &result,
            },
            span,
        )
        .await;
        result
    }

}
