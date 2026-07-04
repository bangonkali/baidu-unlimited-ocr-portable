impl AppState {
    fn parse_page_output(
        &self,
        page_work: &PageWork<'_>,
        ocr: &OcrRunContext<'_>,
    ) -> Result<crate::ocr::ParsedOcrPage> {
        let raw_text = self.run_ocr_or_fallback(
            page_work.image_path,
            &page_work.stream_context(ocr),
            ocr.worker,
        );
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
        if parsed.cleaned_text.is_empty() {
            parsed.cleaned_text.clone_from(&raw_text);
        }
        Ok(parsed)
    }

    async fn complete_page_state(
        &self,
        page_work: &PageWork<'_>,
        parsed: crate::ocr::ParsedOcrPage,
    ) -> Result<CompletedPageUpdate> {
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
            page.cleaned_text = parsed.cleaned_text;
            page.boxes = parsed.boxes;
            page.spans = parsed.spans;
            (
                stored_page(page_work.file_hash, page),
                page.width_px,
                page.height_px,
            )
        };
        let update = CompletedPageUpdate {
            stored,
            page_record: completed_page_record(page_work, width_px, height_px),
            regions_payload: DocumentRegionsPayload {
                file_hash: page_work.file_hash.to_string(),
                boxes: document
                    .pages
                    .iter()
                    .flat_map(|page| page.boxes.clone())
                    .collect(),
            },
            text_payload: DocumentTextPayload {
                file_hash: page_work.file_hash.to_string(),
                pages: started_page_text_records(document),
            },
        };
        drop(state);
        Ok(update)
    }

    async fn persist_completed_page(
        &self,
        page_work: &PageWork<'_>,
        ocr: &OcrRunContext<'_>,
        started: Instant,
        completed: CompletedPageUpdate,
    ) -> Result<()> {
        let elapsed_ms = elapsed_millis_u64(started);
        self.inner
            .repository
            .replace_page_ocr(&completed.stored, ENGINE_ID, ocr.profile_id, elapsed_ms)
            .await?;
        self.increment_run_page(page_work.run_id, page_work.file_hash)
            .await?;
        self.inner
            .hub
            .publish("document.page.changed", completed.page_record);
        self.inner.hub.publish(
            "document.regions.changed",
            serde_json::to_value(completed.regions_payload)?,
        );
        self.inner.hub.publish(
            "document.text.changed",
            serde_json::to_value(completed.text_payload)?,
        );
        self.inner
            .repository
            .upsert_page_metrics(&OcrPageMetrics {
                run_id: page_work.run_id.to_string(),
                file_hash: page_work.file_hash.to_string(),
                page_no: page_work.page_no,
                model_id: ocr.model_id.to_string(),
                runtime_id: ocr.runtime_id.to_string(),
                status: "completed".to_string(),
                token_count: 0,
                avg_tps: 0.0,
                elapsed_ms,
            })
            .await?;
        Ok(())
    }
}

struct CompletedPageUpdate {
    stored: StoredPage,
    page_record: Value,
    regions_payload: DocumentRegionsPayload,
    text_payload: DocumentTextPayload,
}

fn completed_page_record(page_work: &PageWork<'_>, width_px: u32, height_px: u32) -> Value {
    json!({
        "file_hash": page_work.file_hash,
        "page_no": page_work.page_no,
        "status": "completed",
        "width_px": width_px,
        "height_px": height_px,
    })
}
