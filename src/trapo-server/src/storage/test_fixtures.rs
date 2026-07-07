mod test_fixtures {
    use super::*;
    use serde_json::json;

    pub(super) fn run_record(run_id: &str, status: &str) -> StoredRun {
        StoredRun {
            run_id: run_id.to_string(),
            root_path: "dataset".to_string(),
            status: status.to_string(),
            profile_id: "profile".to_string(),
            engine_id: "engine".to_string(),
            model_id: "model".to_string(),
            runtime_id: "runtime".to_string(),
            queued_files: 1,
            processed_pages: 0,
            total_pages: 1,
            error: None,
        }
    }

    pub(super) fn document_record(file_hash: &str, display_name: &str) -> StoredDocument {
        StoredDocument {
            file_hash: file_hash.to_string(),
            display_name: display_name.to_string(),
            extension: "pdf".to_string(),
            size_bytes: 128,
            page_count: 1,
            status: "queued".to_string(),
            error: None,
            root_path: "dataset".to_string(),
            absolute_path: format!("dataset/{display_name}"),
            relative_path: display_name.to_string(),
        }
    }

    pub(super) fn page_record(
        file_hash: &str,
        page_no: u32,
        preview_path: Option<&str>,
    ) -> StoredPage {
        let annotation_id = new_persistence_id();
        StoredPage {
            run_id: None,
            file_hash: file_hash.to_string(),
            page_no,
            width_px: 100,
            height_px: 200,
            render_dpi: 200,
            status: "completed".to_string(),
            error: None,
            preview_path: preview_path.map(str::to_string),
            cleaned_text: "Invoice Total".to_string(),
            raw_text: "Invoice Total".to_string(),
            boxes: vec![OverlayBox {
                annotation_id: annotation_id.clone(),
                region_id: "region-a".to_string(),
                source_region_key: "source-a".to_string(),
                label: "total".to_string(),
                category: "total".to_string(),
                content_markdown: "Invoice Total".to_string(),
                content_html: None,
                page_no,
                left_percent: 10.0,
                top_percent: 20.0,
                width_percent: 30.0,
                height_percent: 40.0,
                hidden: false,
            }],
            spans: vec![TextRegionSpan {
                annotation_id,
                region_id: "region-a".to_string(),
                source_region_key: "source-a".to_string(),
                page_no,
                start: 0,
                end: 13,
            }],
        }
    }

    pub(super) fn download_event(
        event_id: &str,
        download_id: &str,
        event_type: &str,
        total_bytes: Option<u64>,
    ) -> DownloadEventInsert {
        DownloadEventInsert {
            event_id: event_id.to_string(),
            download_id: download_id.to_string(),
            download_key: download_id.to_string(),
            owner_kind: "model".to_string(),
            owner_id: "model-a".to_string(),
            file_id: "model".to_string(),
            file_name: "model.gguf".to_string(),
            target_path: "models/model.gguf".to_string(),
            source_url: "https://example.invalid/model.gguf".to_string(),
            event_type: event_type.to_string(),
            status: "downloading".to_string(),
            downloaded_bytes: 0,
            total_bytes,
            error: None,
            error_kind: None,
            created_at: "2026-07-03T00:00:00Z".to_string(),
        }
    }

    pub(super) fn realtime_event(
        sequence: u64,
        event_type: &str,
        run_id: &str,
        file_hash: &str,
        page_no: u32,
    ) -> StoredRealtimeEvent {
        StoredRealtimeEvent {
            event_id: new_persistence_id(),
            sequence,
            event_type: event_type.to_string(),
            occurred_at: "2026-07-03T00:00:00Z".to_string(),
            run_id: Some(run_id.to_string()),
            file_hash: Some(file_hash.to_string()),
            page_no: Some(page_no),
            payload: json!({"run_id": run_id, "file_hash": file_hash, "page_no": page_no}),
        }
    }

    pub(super) fn work_unit(run_id: &str, file_hash: &str, page_no: u32) -> WorkUnitUpsert {
        WorkUnitUpsert {
            work_unit_id: new_persistence_id(),
            run_id: run_id.to_string(),
            run_engine_id: None,
            work_key: format!("{file_hash}:{page_no}:ocr"),
            file_hash: Some(file_hash.to_string()),
            page_no: Some(page_no),
            phase: "ocr".to_string(),
            engine: "engine".to_string(),
            provider: "local".to_string(),
            model: "model-a".to_string(),
            profile: Some("profile".to_string()),
            execution_key: "ocr".to_string(),
            artifact_variant: Some("source".to_string()),
            metadata: json!({"attempt": 1}),
        }
    }

    pub(super) fn span(
        span_id: &str,
        run_id: &str,
        file_hash: &str,
        page_no: u32,
        status: &str,
        error_message: Option<&str>,
    ) -> DiagnosticSpanInsert {
        DiagnosticSpanInsert {
            span_id: span_id.to_string(),
            trace_id: run_id.to_string(),
            parent_span_id: None,
            task_id: None,
            work_unit_id: None,
            span_kind: "operation".to_string(),
            run_id: Some(run_id.to_string()),
            file_hash: Some(file_hash.to_string()),
            page_no: Some(page_no),
            name: "decode page".to_string(),
            pipeline_step: "ocr".to_string(),
            category: "pipeline".to_string(),
            annotation_engine: Some("engine".to_string()),
            status: status.to_string(),
            started_at: "2026-07-03T00:00:00Z".to_string(),
            ended_at: "2026-07-03T00:00:01Z".to_string(),
            duration_ms: 1000.0,
            attributes: json!({"page": page_no}),
            error_type: error_message.map(|_| "AppError".to_string()),
            error_message: error_message.map(str::to_string),
            error_stack: None,
        }
    }

    pub(super) fn diagnostic_event(
        event_id: &str,
        run_id: &str,
        file_hash: &str,
        page_no: u32,
    ) -> DiagnosticEventInsert {
        DiagnosticEventInsert {
            event_id: event_id.to_string(),
            trace_id: run_id.to_string(),
            span_id: Some("span-error".to_string()),
            run_id: Some(run_id.to_string()),
            file_hash: Some(file_hash.to_string()),
            page_no: Some(page_no),
            timestamp: "2026-07-03T00:00:01Z".to_string(),
            event_type: "error".to_string(),
            name: "decode failed".to_string(),
            severity: "error".to_string(),
            message: "decode failed".to_string(),
            attributes: json!({"page": page_no}),
        }
    }
}
