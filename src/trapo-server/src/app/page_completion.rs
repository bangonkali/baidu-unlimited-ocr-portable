impl AppState {
    fn parse_page_output(
        &self,
        page_work: &PageWork<'_>,
        ocr: &OcrRunContext<'_>,
    ) -> Result<ParsedPageOutput> {
        let raw_text = self.run_ocr_or_fallback(
            page_work.image_path,
            &page_work.stream_context(ocr),
            ocr.worker,
        )?;
        let mut parsed = crate::ocr::parse_ocr_markers(
            &raw_text,
            &crate::ocr::ParseContext {
                file_hash: page_work.file_hash.to_string(),
                page_no: page_work.page_no,
                engine_id: ocr.engine_id.to_string(),
                profile_id: ocr.profile_id.to_string(),
            },
        );
        crate::ocr::apply_region_content(&mut parsed);
        let annotation_drafts = self.assign_annotation_identities(
            page_work.run_id,
            page_work.file_hash,
            ocr.engine_id,
            ocr.profile_id,
            &mut parsed,
        );
        self.write_image_region_snippets(
            page_work.file_hash,
            page_work.image_path,
            &mut parsed.boxes,
        )?;
        Ok(ParsedPageOutput {
            annotation_drafts,
            parsed,
        })
    }

    fn assign_annotation_identities(
        &self,
        run_id: &str,
        file_hash: &str,
        engine_id: &str,
        profile_id: &str,
        parsed: &mut crate::ocr::ParsedOcrPage,
    ) -> Vec<AnnotationIdentityDraft> {
        let boxes = parsed.boxes.clone();
        let mut drafts = Vec::with_capacity(boxes.len());
        for (index, box_record) in boxes.iter().enumerate() {
            let span = parsed
                .spans
                .iter()
                .find(|item| item.source_region_key == box_record.source_region_key);
            let draft = annotation_identity_draft(
                AnnotationDraftScope {
                    run_id,
                    file_hash,
                    engine_id,
                    profile_id,
                    index,
                },
                box_record,
                span,
            );
            let resolved = self.inner.annotation_identities.resolve_and_enqueue(draft);
            apply_annotation_id(parsed, &box_record.source_region_key, &resolved.annotation_id);
            drafts.push(resolved.draft); // skylos: ignore[SKY-D215] annotation drafts store OCR source keys and normalized box data, not filesystem paths.
        }
        drafts
    }

    async fn complete_page_state(
        &self,
        page_work: &PageWork<'_>,
        ocr: &OcrRunContext<'_>,
        output: ParsedPageOutput,
    ) -> Result<CompletedPageUpdate> {
        let ParsedPageOutput {
            annotation_drafts,
            parsed,
        } = output;
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
            annotation_drafts,
            stored,
            page_record: completed_page_record(page_work, width_px, height_px),
            regions_payload: DocumentRegionsPayload {
                file_hash: page_work.file_hash.to_string(),
                run_engine_id: Some(ocr.run_engine_id.to_string()),
                run_id: Some(page_work.run_id.to_string()),
                boxes: document
                    .pages
                    .iter()
                    .flat_map(|page| page.boxes.clone())
                    .collect(),
            },
            text_payload: DocumentTextPayload {
                file_hash: page_work.file_hash.to_string(),
                run_engine_id: Some(ocr.run_engine_id.to_string()),
                run_id: Some(page_work.run_id.to_string()),
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
            .annotation_identities
            .persist_now(&self.inner.repository, &completed.annotation_drafts)
            .await?;
        self.inner
            .repository
            .replace_page_ocr(
                page_work.run_id,
                &completed.stored,
                ocr.engine_id,
                ocr.profile_id,
                elapsed_ms,
            )
            .await?;
        self.inner
            .repository
            .replace_page_output(
                &stored_run_engine_config(&RunEngineConfigState {
                    run_engine_id: ocr.run_engine_id.to_string(),
                    run_id: page_work.run_id.to_string(),
                    ordinal: 0,
                    engine_kind: ocr.engine_kind.to_string(),
                    engine_id: ocr.engine_id.to_string(),
                    model_id: (!ocr.model_id.is_empty()).then(|| ocr.model_id.to_string()),
                    profile_id: (!ocr.profile_id.is_empty()).then(|| ocr.profile_id.to_string()),
                    runtime_id: (!ocr.runtime_id.is_empty()).then(|| ocr.runtime_id.to_string()),
                    parameters: json!({}),
                    status: "running".to_string(),
                    error: None,
                    usable_output_count: 0,
                }),
                &completed.stored,
                Some(page_work.work_unit_id),
                if ocr.engine_kind == "document_understanding" {
                    "markdown"
                } else {
                    "ocr"
                },
                elapsed_ms,
            )
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

#[derive(Clone, Copy)]
struct AnnotationDraftScope<'a> {
    run_id: &'a str,
    file_hash: &'a str,
    engine_id: &'a str,
    profile_id: &'a str,
    index: usize,
}

fn annotation_identity_draft(
    scope: AnnotationDraftScope<'_>,
    box_record: &crate::workbench_types::OverlayBox,
    span: Option<&crate::workbench_types::TextRegionSpan>,
) -> AnnotationIdentityDraft {
    AnnotationIdentityDraft {
        annotation_id: None,
        run_id: scope.run_id.to_string(),
        file_hash: scope.file_hash.to_string(),
        page_no: box_record.page_no,
        engine_id: scope.engine_id.to_string(),
        profile_id: scope.profile_id.to_string(),
        source_region_key: box_record.source_region_key.clone(),
        discovery_index: u32::try_from(scope.index).unwrap_or(u32::MAX),
        label: box_record.label.clone(),
        category: box_record.category.clone(),
        x1: box_record.left_percent / 100.0 * 999.0,
        y1: box_record.top_percent / 100.0 * 999.0,
        x2: (box_record.left_percent + box_record.width_percent) / 100.0 * 999.0,
        y2: (box_record.top_percent + box_record.height_percent) / 100.0 * 999.0,
        geometry: box_record.geometry.clone(),
        span_start: span.map_or(0, |item| item.start),
        span_end: span.map_or(0, |item| item.end),
        content_markdown: box_record.content_markdown.clone(),
        content_html: box_record.content_html.clone(),
    }
}

fn apply_annotation_id(
    parsed: &mut crate::ocr::ParsedOcrPage,
    source_region_key: &str,
    annotation_id: &str,
) {
    for span in parsed
        .spans
        .iter_mut()
        .filter(|item| item.source_region_key == source_region_key)
    {
        span.annotation_id = annotation_id.to_string();
        span.region_id = annotation_id.to_string();
    }
    for box_record in parsed
        .boxes
        .iter_mut()
        .filter(|item| item.source_region_key == source_region_key)
    {
        box_record.annotation_id = annotation_id.to_string();
        box_record.region_id = annotation_id.to_string();
    }
}

struct ParsedPageOutput {
    annotation_drafts: Vec<AnnotationIdentityDraft>,
    parsed: crate::ocr::ParsedOcrPage,
}

struct CompletedPageUpdate {
    annotation_drafts: Vec<AnnotationIdentityDraft>,
    stored: StoredPage,
    page_record: Value,
    regions_payload: DocumentRegionsPayload,
    text_payload: DocumentTextPayload,
}

fn completed_page_record(page_work: &PageWork<'_>, width_px: u32, height_px: u32) -> Value {
    json!({
        "run_id": page_work.run_id,
        "file_hash": page_work.file_hash,
        "page_no": page_work.page_no,
        "status": "completed",
        "width_px": width_px,
        "height_px": height_px,
        "preview_available": true,
    })
}
