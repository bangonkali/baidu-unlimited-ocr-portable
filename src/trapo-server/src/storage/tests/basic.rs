mod tests {
    use super::*;

    #[tokio::test]
    async fn migrates_and_persists_settings() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        repo.put_setting("selected_model_id", &Value::String("model".to_string()))
            .await?;
        assert_eq!(
            repo.setting_value("selected_model_id").await?,
            Some(Value::String("model".to_string()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn reloads_page_regions_and_spans() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        let annotation_id = crate::ids::new_persistence_id();
        let region_id = "src_region-a".to_string();
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
                annotation_id: annotation_id.clone(),
                region_id: region_id.clone(),
                source_region_key: "source-a".to_string(),
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
                annotation_id: annotation_id.clone(),
                region_id: region_id.clone(),
                source_region_key: "source-a".to_string(),
                page_no: 1,
                start: 0,
                end: 5,
            }],
        };

        repo.upsert_page(&page).await?;
        repo.replace_page_ocr(&page, "engine", "profile", 42)
            .await?;

        let snapshot = repo.load_snapshot().await?;
        assert_eq!(snapshot.pages.len(), 1);
        let loaded = &snapshot.pages[0];
        assert_eq!(loaded.boxes.len(), 1);
        assert_eq!(loaded.spans.len(), 1);
        assert!(crate::ids::is_uuid_v7(&loaded.boxes[0].annotation_id));
        assert_eq!(loaded.boxes[0].annotation_id, annotation_id);
        assert_eq!(loaded.boxes[0].region_id, region_id);
        assert_eq!(loaded.spans[0].annotation_id, annotation_id);
        assert_eq!(loaded.spans[0].region_id, region_id);
        Ok(())
    }

    #[tokio::test]
    async fn persists_discovered_annotation_identity_before_text_completion() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        let annotation_id = new_persistence_id();
        let draft = annotation_draft(annotation_id.clone(), new_persistence_id(), 2, 0);

        repo.persist_discovered_annotations(vec![draft.clone()])
            .await?;
        assert!(is_uuid_v7(&annotation_id));

        let mut updated = draft;
        updated.annotation_id = Some(new_persistence_id());
        updated.label = "Invoice total updated".to_string();
        updated.content_markdown = "Invoice total updated".to_string();
        updated.x2 = 55.0;
        repo.persist_discovered_annotations(vec![updated]).await?;

        let query_annotation_id = annotation_id.clone();
        let (identity_count, identity_label, created_at, region_count, link_count) = repo
            .with_read(move |conn| {
                let identity_count = conn.query_row(
                    "SELECT count(*) FROM document_annotation_identities
                     WHERE annotation_id = ? AND source_region_key = ?",
                    params![query_annotation_id.as_str(), "pdfium:file-a:2:0"],
                    |row| row.get::<_, i64>(0),
                )?;
                let (identity_label, created_at) = conn.query_row(
                    "SELECT label, CAST(created_at AS VARCHAR)
                     FROM document_annotation_identities WHERE annotation_id = ?",
                    params![query_annotation_id.as_str()],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )?;
                let region_count = conn.query_row(
                    "SELECT count(*) FROM document_regions
                     WHERE region_id = ? AND annotation_id = ? AND content_markdown = ?",
                    params![
                        query_annotation_id.as_str(),
                        query_annotation_id.as_str(),
                        "Invoice total updated"
                    ],
                    |row| row.get::<_, i64>(0),
                )?;
                let link_count = conn.query_row(
                    "SELECT count(*) FROM document_text_region_links
                     WHERE annotation_id = ? AND text_start = ? AND text_end = ?",
                    params![query_annotation_id.as_str(), 15_i64, 28_i64],
                    |row| row.get::<_, i64>(0),
                )?;
                Ok((
                    identity_count,
                    identity_label,
                    created_at,
                    region_count,
                    link_count,
                ))
            })
            .await?;

        assert_eq!(identity_count, 1);
        assert_eq!(identity_label, "Invoice total updated");
        assert!(!created_at.is_empty());
        assert_eq!(region_count, 1);
        assert_eq!(link_count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn persists_preassigned_annotation_identity_batches() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        repo.persist_discovered_annotations(Vec::new()).await?;
        let annotation_id = new_persistence_id();
        let run_id = new_persistence_id();
        let draft = annotation_draft(annotation_id.clone(), run_id.clone(), 3, 0);
        let mut updated = draft.clone();
        updated.label = "Total due".to_string();
        updated.content_markdown = "Total due".to_string();
        let second_id = new_persistence_id();
        let mut second = annotation_draft(second_id.clone(), run_id.clone(), 3, 1);
        second.label = "Date".to_string();

        repo.persist_discovered_annotations(vec![draft, updated, second])
            .await?;

        let (identity_count, updated_label, region_count) = repo
            .with_read(move |conn| {
                let identity_count = conn.query_row(
                    "SELECT count(*) FROM document_annotation_identities WHERE run_id = ?",
                    params![run_id],
                    |row| row.get::<_, i64>(0),
                )?;
                let updated_label = conn.query_row(
                    "SELECT label FROM document_annotation_identities WHERE annotation_id = ?",
                    params![annotation_id.as_str()],
                    |row| row.get::<_, String>(0),
                )?;
                let region_count = conn.query_row(
                    "SELECT count(*) FROM document_regions WHERE annotation_id IN (?, ?)",
                    params![annotation_id.as_str(), second_id.as_str()],
                    |row| row.get::<_, i64>(0),
                )?;
                Ok((identity_count, updated_label, region_count))
            })
            .await?;

        assert_eq!(identity_count, 2);
        assert_eq!(updated_label, "Total due");
        assert_eq!(region_count, 2);
        Ok(())
    }

    #[tokio::test]
    async fn persists_and_lists_ocr_stream_events() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
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
        )
        .await?;

        let events = repo
            .list_ocr_stream_events(Some("run-a"), Some("file-a"), Some(1), None, 10)
            .await?;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 7);
        assert_eq!(events[0].payload["text"], "Total");
        Ok(())
    }

    #[tokio::test]
    async fn persists_download_lifecycle_events() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        let event = DownloadEventInsert {
            event_id: "event-a".to_string(),
            download_id: "model:model-a:model".to_string(),
            download_key: "model:model-a:model".to_string(),
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

        repo.insert_download_event(&event).await?;

        assert_eq!(
            repo.download_event_count(&event.download_id, "started")
                .await?,
            1
        );
        Ok(())
    }

    #[tokio::test]
    async fn reloads_run_document_membership() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
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
        })
        .await?;
        repo.replace_run_documents("run-a", &["file-b".to_string(), "file-a".to_string()])
            .await?;

        let snapshot = repo.load_snapshot().await?;

        let files: Vec<_> = snapshot
            .run_documents
            .iter()
            .map(|item| item.file_hash.as_str())
            .collect();
        assert_eq!(files, ["file-b", "file-a"]);
        Ok(())
    }

    fn annotation_draft(
        annotation_id: String,
        run_id: String,
        page_no: u32,
        discovery_index: u32,
    ) -> AnnotationIdentityDraft {
        AnnotationIdentityDraft {
            annotation_id: Some(annotation_id),
            run_id,
            file_hash: "file-a".to_string(),
            page_no,
            engine_id: "pdfium-unlimited-ocr".to_string(),
            profile_id: "default".to_string(),
            source_region_key: format!("pdfium:file-a:{page_no}:{discovery_index}"),
            discovery_index,
            label: "Invoice total".to_string(),
            x1: 10.0,
            y1: 20.0,
            x2: 50.0,
            y2: 30.0,
            span_start: 15,
            span_end: 28,
            content_markdown: "Invoice total".to_string(),
            content_html: None,
        }
    }
}
