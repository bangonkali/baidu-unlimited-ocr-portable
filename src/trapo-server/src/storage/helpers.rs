fn metrics_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<OcrPageMetrics> {
    Ok(OcrPageMetrics {
        run_id: row.get(0)?,
        file_hash: row.get(1)?,
        page_no: i64_to_u32(row.get::<_, i64>(2)?),
        model_id: row.get(3)?,
        runtime_id: row.get(4)?,
        status: row.get(5)?,
        token_count: row.get::<_, i64>(6)? as u64,
        avg_tps: row.get(7)?,
        elapsed_ms: row.get::<_, i64>(8)? as u64,
    })
}

fn collect_rows<T>(
    rows: duckdb::MappedRows<'_, impl FnMut(&duckdb::Row<'_>) -> duckdb::Result<T>>,
) -> Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

fn i64_to_u32(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value.max(0)).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_and_persists_settings() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        repo.put_setting("selected_model_id", &Value::String("model".to_string()))?;
        assert_eq!(
            repo.setting_value("selected_model_id")?,
            Some(Value::String("model".to_string()))
        );
        Ok(())
    }

    #[test]
    fn reloads_page_regions_and_spans() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        let page = StoredPage {
            file_hash: "file-a".to_string(),
            page_no: 1,
            width_px: 100,
            height_px: 200,
            render_dpi: 200,
            status: "completed".to_string(),
            error: None,
            preview_path: Some("page.png".to_string()),
            cleaned_text: "Total".to_string(),
            raw_text: "Total".to_string(),
            boxes: vec![OverlayBox {
                region_id: "region-a".to_string(),
                label: "Total".to_string(),
                content_markdown: "Total".to_string(),
                content_html: None,
                page_no: 1,
                left_percent: 10.0,
                top_percent: 20.0,
                width_percent: 30.0,
                height_percent: 40.0,
                hidden: false,
            }],
            spans: vec![TextRegionSpan {
                region_id: "region-a".to_string(),
                page_no: 1,
                start: 0,
                end: 5,
            }],
        };

        repo.upsert_page(&page)?;
        repo.replace_page_ocr(&page, "engine", "profile", 42)?;

        let snapshot = repo.load_snapshot()?;
        assert_eq!(snapshot.pages.len(), 1);
        let loaded = &snapshot.pages[0];
        assert_eq!(loaded.boxes.len(), 1);
        assert_eq!(loaded.spans.len(), 1);
        assert_eq!(loaded.spans[0].region_id, "region-a");
        Ok(())
    }

    #[test]
    fn persists_and_lists_ocr_stream_events() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        repo.persist_realtime_event(
            7,
            "ocr.page.text.patch",
            "2026-07-03T00:00:00Z",
            &serde_json::json!({
                "run_id": "run-a",
                "file_hash": "file-a",
                "page_no": 1,
                "text": "Total"
            }),
        )?;

        let events = repo.list_ocr_stream_events(Some("run-a"), Some("file-a"), Some(1), None, 10)?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 7);
        assert_eq!(events[0].payload["text"], "Total");
        Ok(())
    }

    #[test]
    fn persists_download_lifecycle_events() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        let event = DownloadEventInsert {
            event_id: "event-a".to_string(),
            download_id: "model:model-a:model".to_string(),
            owner_kind: "model".to_string(),
            owner_id: "model-a".to_string(),
            file_id: "model".to_string(),
            file_name: "model.gguf".to_string(),
            target_path: "models/model.gguf".to_string(),
            source_url: "https://example.invalid/model.gguf".to_string(),
            event_type: "started".to_string(),
            status: "downloading".to_string(),
            downloaded_bytes: 0,
            total_bytes: Some(128),
            error: None,
            created_at: "2026-07-03T00:00:00Z".to_string(),
        };

        repo.insert_download_event(&event)?;

        assert_eq!(repo.download_event_count(&event.download_id, "started")?, 1);
        Ok(())
    }

    #[test]
    fn reloads_run_document_membership() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        repo.upsert_run(&StoredRun {
            run_id: "run-a".to_string(),
            root_path: "dataset".to_string(),
            status: "queued".to_string(),
            profile_id: "profile".to_string(),
            engine_id: "engine".to_string(),
            model_id: "model".to_string(),
            runtime_id: "runtime".to_string(),
            queued_files: 2,
            processed_pages: 0,
            total_pages: 2,
            error: None,
        })?;
        repo.replace_run_documents("run-a", &["file-b".to_string(), "file-a".to_string()])?;

        let snapshot = repo.load_snapshot()?;

        let files: Vec<_> = snapshot
            .run_documents
            .iter()
            .map(|item| item.file_hash.as_str())
            .collect();
        assert_eq!(files, ["file-b", "file-a"]);
        Ok(())
    }
}
