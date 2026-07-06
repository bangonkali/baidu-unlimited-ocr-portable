mod coverage_tests {
    use super::*;
    use super::test_fixtures::*;
    use serde_json::json;

    const I64_MAX_AS_U64: u64 = 9_223_372_036_854_775_807;

    #[tokio::test]
    async fn core_document_page_and_metric_queries_cover_empty_updates_and_limits() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("nested").join("trapo.duckdb")).await?;
        assert!(repo.path().starts_with(temp.path()));

        assert_eq!(repo.setting_value("missing").await?, None);
        repo.put_setting("workbench_ui", &json!({"theme": "dark"}))
            .await?;
        repo.put_setting("workbench_ui", &json!({"theme": "light"}))
            .await?;
        assert_eq!(
            repo.setting_value("workbench_ui").await?,
            Some(json!({"theme": "light"}))
        );

        let mut run = run_record("run-a", "queued");
        repo.upsert_run(&run).await?;
        run.status = "completed".to_string();
        run.processed_pages = 1;
        repo.upsert_run(&run).await?;
        repo.replace_run_documents("missing-run", &[]).await?;

        let mut document = document_record("file-a", "Invoice.pdf");
        repo.upsert_document(&document).await?;
        document.display_name = "Invoice Total.pdf".to_string();
        document.status = "completed".to_string();
        repo.upsert_document(&document).await?;
        repo.replace_run_documents("run-a", &["file-a".to_string(), "file-b".to_string()])
            .await?;

        let page = page_record("file-a", 1, Some("cache/file-a/page-1.png"));
        repo.upsert_page(&page).await?;
        repo.replace_page_ocr("run-a", &page, "engine", "profile", 42)
            .await?;

        assert_eq!(
            repo.search_document_hashes("total", 10).await?,
            vec!["file-a".to_string()]
        );
        assert!(repo.search_document_hashes("missing", 10).await?.is_empty());
        assert!(repo.search_document_hashes("total", 0).await?.is_empty());

        let mut snapshot = repo.load_snapshot().await?;
        assert_eq!(snapshot.runs.len(), 1);
        assert_eq!(snapshot.documents.len(), 1);
        assert_eq!(snapshot.run_documents.len(), 2);
        assert_eq!(snapshot.pages.len(), 1);
        assert_eq!(snapshot.pages[0].boxes.len(), 1);
        assert_eq!(snapshot.pages[0].spans.len(), 1);

        let empty_page = StoredPage {
            preview_path: None,
            boxes: Vec::new(),
            spans: Vec::new(),
            ..page_record("file-a", 1, None)
        };
        repo.upsert_page(&empty_page).await?;
        repo.replace_page_ocr("run-a", &empty_page, "engine", "profile", 0)
            .await?;
        snapshot = repo.load_snapshot().await?;
        assert_eq!(snapshot.pages[0].preview_path, Some("cache/file-a/page-1.png".to_string()));
        assert!(snapshot.pages[0].boxes.is_empty());
        assert!(snapshot.pages[0].spans.is_empty());

        assert!(repo.list_page_metrics(Some("missing-run"), 10).await?.is_empty());
        repo.upsert_page_metrics(&OcrPageMetrics {
            run_id: "run-a".to_string(),
            file_hash: "file-a".to_string(),
            page_no: 1,
            model_id: "model-a".to_string(),
            runtime_id: "runtime-a".to_string(),
            status: "running".to_string(),
            token_count: u64::MAX,
            avg_tps: 12.5,
            elapsed_ms: u64::MAX,
        })
        .await?;
        assert!(repo.list_page_metrics(Some("run-a"), 0).await?.is_empty());
        let metrics = repo.list_page_metrics(Some("run-a"), 1).await?;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].token_count, I64_MAX_AS_U64);
        assert_eq!(metrics[0].elapsed_ms, I64_MAX_AS_U64);
        Ok(())
    }

    #[tokio::test]
    async fn realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits()
    -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        repo.upsert_run(&run_record("run-d", "running")).await?;
        repo.upsert_document(&document_record("file-d", "Diagnostics.pdf"))
            .await?;

        assert_download_queries(&repo).await?;
        assert_realtime_queries(&repo).await?;
        assert_diagnostic_queries(&repo).await
    }

    async fn assert_download_queries(repo: &Repository) -> Result<()> {
        let download = download_event("event-a", "download-a", "started", None);
        repo.insert_download_event(&download).await?;
        repo.insert_download_event(&DownloadEventInsert {
            event_type: "failed".to_string(),
            status: "failed".to_string(),
            ..download.clone()
        })
        .await?;
        assert_eq!(repo.download_event_count("download-a", "started").await?, 1);
        assert_eq!(repo.download_event_count("download-a", "failed").await?, 0);
        assert_eq!(repo.download_event_count("missing", "started").await?, 0);
        Ok(())
    }

    async fn assert_realtime_queries(repo: &Repository) -> Result<()> {
        repo.persist_realtime_events(vec![
            realtime_event(1, "model.changed", "run-d", "file-d", 1),
            realtime_event(1, "ocr.page.text.patch", "run-d", "file-d", 1),
            realtime_event(2, "ocr.page.completed", "run-d", "file-d", 2),
        ])
        .await?;
        assert_eq!(
            repo.list_ocr_stream_events(None, None, None, None, 0)
                .await?
                .len(),
            1
        );
        assert_eq!(
            repo.list_ocr_stream_events(Some("run-d"), Some("file-d"), None, None, 10)
                .await?
                .len(),
            2
        );
        assert_eq!(
            repo.list_ocr_stream_events(Some("run-d"), None, Some(1), None, 10)
                .await?
                .len(),
            1
        );
        assert!(
            repo.list_ocr_stream_events(Some("missing"), None, None, None, 10)
                .await?
                .is_empty()
        );
        assert_eq!(
            repo.list_ocr_stream_events(None, None, None, Some(1), 10)
                .await?[0]
                .sequence,
            2
        );
        Ok(())
    }

    async fn assert_diagnostic_queries(repo: &Repository) -> Result<()> {
        seed_diagnostic_queries(repo).await?;
        assert_diagnostic_read_queries(repo).await
    }

    async fn seed_diagnostic_queries(repo: &Repository) -> Result<()> {
        let unit = work_unit("run-d", "file-d", 1);
        repo.upsert_work_unit(&unit).await?;
        repo.start_work_unit("run-d", "file-d:1:ocr").await?;
        repo.start_work_unit("missing", "missing").await?;
        repo.finish_work_unit(
            "run-d",
            "file-d:1:ocr",
            "completed",
            &json!({"pages": 1}),
            None,
        )
        .await?;
        repo.finish_work_unit("missing", "missing", "completed", &json!({}), None)
            .await?;
        repo.insert_diagnostic_span(&span("span-ok", "run-d", "file-d", 1, "ok", None))
            .await?;
        repo.insert_diagnostic_span(&span(
            "span-error",
            "run-d",
            "file-d",
            2,
            "error",
            Some("decode failed"),
        ))
        .await?;
        repo.insert_diagnostic_event(&diagnostic_event("event-d", "run-d", "file-d", 2))
            .await?;
        repo.insert_model_lease(&DiagnosticModelLeaseInsert {
            run_id: "run-d".to_string(),
            execution_key: "model".to_string(),
            provider: "local".to_string(),
            model: "model-a".to_string(),
            status: "ok".to_string(),
            metadata: json!({"context": 4096}),
        })
        .await?;
        repo.insert_model_lease(&DiagnosticModelLeaseInsert {
            status: "fallback".to_string(),
            metadata: json!({"context": 2048}),
            run_id: "run-d".to_string(),
            execution_key: "model".to_string(),
            provider: "local".to_string(),
            model: "model-a".to_string(),
        })
        .await?;
        Ok(())
    }

    async fn assert_diagnostic_read_queries(repo: &Repository) -> Result<()> {
        assert_eq!(repo.diagnostic_runs(0).await?.len(), 1);
        let (spans, events) = repo
            .diagnostic_trace(&DiagnosticTraceFilter {
                run_id: Some("run-d"),
                file_hash: Some("file-d"),
                page_no: Some(2),
                status: Some("error"),
                q: Some("decode"),
                limit: 0,
            })
            .await?;
        assert_eq!(spans.len(), 1);
        assert_eq!(events.len(), 1);
        let (spans, events) = repo
            .diagnostic_trace(&DiagnosticTraceFilter {
                run_id: Some("run-d"),
                file_hash: None,
                page_no: None,
                status: None,
                q: Some("not-present"),
                limit: 10,
            })
            .await?;
        assert!(spans.is_empty());
        assert!(events.is_empty());
        assert_eq!(
            repo.diagnostic_work_units(Some("run-d"), 0).await?[0].status,
            "completed"
        );
        assert!(
            repo.diagnostic_work_units(Some("missing"), 10)
                .await?
                .is_empty()
        );
        assert_eq!(
            repo.diagnostic_model_leases(Some("run-d"), 0).await?[0].status,
            "fallback"
        );
        assert!(
            repo.diagnostic_model_leases(Some("missing"), 10)
                .await?
                .is_empty()
        );
        Ok(())
    }
}
