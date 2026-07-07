mod engine_output_tests {
    use super::test_fixtures::*;
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn engine_configs_and_outputs_cover_preview_queries() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;
        repo.upsert_run(&run_record("run-engine", "running")).await?;
        repo.upsert_document(&document_record("file-engine", "Engines.pdf"))
            .await?;
        let page = page_record("file-engine", 1, Some("cache/file-engine/page-1.png"));
        repo.upsert_page(&page).await?;

        let config_id = new_persistence_id();
        let config = StoredRunEngineConfig {
            run_engine_id: config_id.clone(),
            run_id: "run-engine".to_string(),
            ordinal: 0,
            engine_kind: "ocr".to_string(),
            engine_id: "unlimited-ocr-ffi".to_string(),
            model_id: Some("unlimited-ocr-q4-k-m".to_string()),
            profile_id: Some("experimental-exact-prefill-q4".to_string()),
            runtime_id: Some("runtime-a".to_string()),
            parameters: json!({"temperature": 0}),
            status: "queued".to_string(),
            error: None,
            usable_output_count: 0,
        };
        repo.replace_run_engine_configs("run-engine", std::slice::from_ref(&config))
            .await?;
        repo.update_run_engine_config_status(&config_id, "completed", None, 1)
            .await?;
        let mut completed = config;
        completed.status = "completed".to_string();
        completed.usable_output_count = 1;
        repo.replace_page_output(&completed, &page, Some(&new_persistence_id()), "ocr", 12)
            .await?;

        assert!(is_uuid_v7(&config_id));
        assert!(
            repo.preview_results_for_document("run-engine", "missing-file")
                .await?
                .is_empty()
        );
        let results = repo
            .preview_results_for_document("run-engine", "file-engine")
            .await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].run_engine_id, config_id);
        assert_eq!(results[0].output_count, 1);
        assert_eq!(results[0].page_count, 1);
        assert_eq!(
            results[0].provenance.as_ref().and_then(|value| value["engine_id"].as_str()),
            Some("unlimited-ocr-ffi")
        );
        assert_eq!(
            results[0].provenance.as_ref().and_then(|value| value["runtime_id"].as_str()),
            Some("runtime-a")
        );

        let text = repo
            .load_document_text_for_run_engine("file-engine", &results[0].run_engine_id)
            .await?;
        assert_eq!(text.len(), 1);
        assert_eq!(text[0].spans.len(), 1);
        let boxes = repo
            .load_document_regions_for_run_engine("file-engine", &results[0].run_engine_id)
            .await?;
        assert_eq!(boxes.len(), 1);
        assert!(
            repo.load_document_text_for_run_engine("file-engine", "missing-engine")
                .await?
                .is_empty()
        );

        repo.replace_run_engine_configs("run-engine", &[]).await?;
        assert!(
            repo.preview_results_for_document("run-engine", "file-engine")
                .await?
                .is_empty()
        );
        Ok(())
    }
}
