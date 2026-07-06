#[cfg(test)]
mod rag_tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn pipeline_task_allows_only_one_active_task() -> Result<()> {
        let repo = temp_repo().await?;
        let first = repo
            .create_pipeline_task("text_index", Some("run-a"), &json!({"run": "run-a"}))
            .await?;
        assert!(is_uuid_v7(&first.task_id));
        let conflict = repo
            .create_pipeline_task("generate_embedding", Some("run-a"), &json!({}))
            .await;
        assert!(matches!(conflict, Err(AppError::Conflict(_))));

        let started = repo.start_pipeline_task(&first.task_id, "runner-a").await?;
        assert_eq!(started.status, "running");
        assert_eq!(started.runner_id.as_deref(), Some("runner-a"));
        let finished = repo
            .finish_pipeline_task(&first.task_id, "completed", &json!({"ok": true}), None)
            .await?;
        assert_eq!(finished.status, "completed");
        assert!(repo.active_pipeline_task().await?.is_none());
        let diagnostic_tasks = repo
            .pipeline_tasks_for_diagnostics(Some("run-a"), 10)
            .await?;
        assert_eq!(diagnostic_tasks.len(), 1);
        assert_eq!(diagnostic_tasks[0].task_id, first.task_id);
        Ok(())
    }

    #[tokio::test]
    async fn embedding_models_and_used_models_round_trip() -> Result<()> {
        let repo = temp_repo().await?;
        let model = embedding_model("model-a", 128);
        repo.upsert_rag_embedding_models(std::slice::from_ref(&model))
            .await?;
        let models = repo.list_rag_embedding_models().await?;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "model-a");
        assert_eq!(models[0].dimension, 128);

        repo.upsert_rag_embedding_run(&RagEmbeddingRunRow {
            embedding_run_id: new_persistence_id(),
            task_id: None,
            source_run_id: "run-a".to_string(),
            model_id: "model-a".to_string(),
            requested_dimension: 128,
            actual_dimension: 128,
            status: "completed".to_string(),
            segments_total: 1,
            segments_embedded: 1,
            started_at: "2026-07-06T00:00:00Z".to_string(),
            finished_at: Some("2026-07-06T00:00:01Z".to_string()),
            error: None,
            params: json!({}),
        })
        .await?;
        let used = repo.list_used_rag_embedding_models().await?;
        assert_eq!(used.len(), 1);
        assert_eq!(used[0].model_id, "model-a");
        Ok(())
    }

    #[tokio::test]
    async fn text_segments_index_and_fts_search_round_trip() -> Result<()> {
        let repo = temp_repo().await?;
        let segments = vec![RagTextSegmentRow {
            segment_id: new_persistence_id(),
            source_run_id: "run-a".to_string(),
            file_hash: "file-a".to_string(),
            page_no: 1,
            segment_index: 0,
            annotation_id: Some(new_persistence_id()),
            category: "page_text".to_string(),
            text: "duckdb vector search invoice total".to_string(),
            token_estimate: 6,
            text_start: 0,
            text_end: 35,
            source_kind: "page".to_string(),
        }];
        assert_eq!(
            repo.replace_rag_text_segments("run-a", &segments).await?,
            1
        );
        let loaded = repo.load_rag_text_segments("run-a").await?;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, segments[0].text);

        let text_index_run_id = new_persistence_id();
        repo.upsert_rag_text_index_run(&RagTextIndexRunRow {
            text_index_run_id: text_index_run_id.clone(),
            task_id: None,
            source_run_id: "run-a".to_string(),
            status: "running".to_string(),
            segments_indexed: 1,
            started_at: "2026-07-06T00:00:00Z".to_string(),
            finished_at: None,
            error: None,
        })
        .await?;
        assert_eq!(
            repo.refresh_rag_fts_index(&text_index_run_id, "run-a")
                .await?,
            1
        );
        let hits = repo.rag_fts_search("invoice", Some("run-a"), 10).await?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].file_hash, "file-a");
        Ok(())
    }

    #[tokio::test]
    async fn embedding_vectors_and_vss_search_round_trip() -> Result<()> {
        let repo = temp_repo().await?;
        let model = embedding_model("model-a", 128);
        repo.upsert_rag_embedding_models(&[model]).await?;
        let segment = RagTextSegmentRow {
            segment_id: new_persistence_id(),
            source_run_id: "run-a".to_string(),
            file_hash: "file-a".to_string(),
            page_no: 1,
            segment_index: 0,
            annotation_id: None,
            category: "page_text".to_string(),
            text: "semantic invoice total".to_string(),
            token_estimate: 3,
            text_start: 0,
            text_end: 22,
            source_kind: "page".to_string(),
        };
        repo.replace_rag_text_segments("run-a", std::slice::from_ref(&segment))
            .await?;
        let embedding_run_id = new_persistence_id();
        repo.upsert_rag_embedding_run(&RagEmbeddingRunRow {
            embedding_run_id: embedding_run_id.clone(),
            task_id: None,
            source_run_id: "run-a".to_string(),
            model_id: "model-a".to_string(),
            requested_dimension: 128,
            actual_dimension: 128,
            status: "running".to_string(),
            segments_total: 1,
            segments_embedded: 0,
            started_at: "2026-07-06T00:00:00Z".to_string(),
            finished_at: None,
            error: None,
            params: json!({}),
        })
        .await?;
        let vector = unit_vector(128, 0);
        let inserted = repo
            .insert_rag_embedding_vectors(
                128,
                &[RagEmbeddingVectorRow {
                    embedding_run_id,
                    source_run_id: "run-a".to_string(),
                    segment_id: segment.segment_id,
                    model_id: "model-a".to_string(),
                    file_hash: "file-a".to_string(),
                    page_no: 1,
                    embedding: vector.clone(),
                    metadata: json!({"kind": "test"}),
                }],
            )
            .await?;
        assert_eq!(inserted, 1);
        let hits = repo
            .rag_vss_search(&vector, 128, "model-a", Some("run-a"), 10)
            .await?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].hit_source, "vss");
        assert!(hits[0].score > 0.99);
        Ok(())
    }

    async fn temp_repo() -> Result<Repository> {
        let temp = tempfile::tempdir()?;
        Repository::open(temp.keep().join("trapo.duckdb")).await
    }

    fn embedding_model(model_id: &str, dimension: u32) -> RagEmbeddingModelRow {
        RagEmbeddingModelRow {
            model_id: model_id.to_string(),
            display_name: "Test embedding".to_string(),
            provider: "Test".to_string(),
            repo_id: "example/repo".to_string(),
            filename: "model.gguf".to_string(),
            revision: "main".to_string(),
            routing_origin: "embedding".to_string(),
            model_family: "test".to_string(),
            dimension,
            context_tokens: 512,
            pooling: "mean".to_string(),
            normalize: true,
            query_prefix: "query: ".to_string(),
            document_prefix: "document: ".to_string(),
            llama_params: json!({"n_gpu_layers": 0}),
            recommended_vram_gb: 1.0,
            active: true,
        }
    }

    fn unit_vector(dimension: usize, hot_index: usize) -> Vec<f32> {
        let mut vector = vec![0.0; dimension];
        vector[hot_index] = 1.0;
        vector
    }
}
