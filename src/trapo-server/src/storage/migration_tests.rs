mod migration_tests {
    use super::*;

    #[tokio::test]
    async fn uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb")).await?;

        repo.with_write(|conn| {
            seed_legacy_rows(&conn)?;
            Repository::migrate_generated_ids_to_uuid_v7(&conn)?;
            let mappings_after_first_run = migration_count(&conn)?;
            Repository::migrate_generated_ids_to_uuid_v7(&conn)?;

            assert_eq!(migration_count(&conn)?, mappings_after_first_run);
            let run_id = text_value(&conn, "SELECT run_id FROM ingest_runs LIMIT 1")?;
            let trace_id = text_value(
                &conn,
                "SELECT trace_id FROM ingest_diagnostic_spans LIMIT 1",
            )?;
            let region_id = text_value(&conn, "SELECT region_id FROM document_regions LIMIT 1")?;
            let annotation_id = text_value(
                &conn,
                "SELECT annotation_id FROM document_annotation_identities LIMIT 1",
            )?;
            let download_id =
                text_value(&conn, "SELECT download_id FROM download_events LIMIT 1")?;
            let event_id = text_value(&conn, "SELECT event_id FROM ocr_stream_events LIMIT 1")?;

            for id in [&run_id, &trace_id, &region_id, &annotation_id, &download_id, &event_id] {
                assert!(is_uuid_v7(id), "{id} should be a UUID v7");
            }
            assert_eq!(run_id, trace_id);
            assert_eq!(region_id, annotation_id);
            assert_eq!(
                text_value(&conn, "SELECT run_id FROM ocr_page_metrics LIMIT 1")?,
                run_id
            );
            assert_eq!(
                text_value(&conn, "SELECT annotation_id FROM document_text_region_links LIMIT 1")?,
                region_id
            );
            assert_eq!(
                text_value(&conn, "SELECT region_id FROM annotation_visibility_overrides LIMIT 1")?,
                region_id
            );

            let payload = text_value(&conn, "SELECT payload_json FROM ocr_stream_events LIMIT 1")?;
            let markdown =
                text_value(&conn, "SELECT content_markdown FROM document_regions LIMIT 1")?;
            assert!(!payload.contains("legacy-run"));
            assert!(!markdown.contains("legacy-region"));
            assert!(payload.contains(&run_id));
            assert!(markdown.contains(&region_id));
            Ok(())
        })
        .await
    }

    fn seed_legacy_rows(conn: &Connection) -> Result<()> {
        seed_legacy_document_rows(conn)?;
        seed_legacy_diagnostic_rows(conn)?;
        seed_legacy_download_and_replay_rows(conn)
    }

    fn seed_legacy_document_rows(conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO ingest_runs(run_id, root_path, status, profile_id, engine_id, reprocess,
             queued_files, processed_pages, total_pages, model_id, runtime_id)
             VALUES ('legacy-run', 'dataset', 'completed', 'profile', 'engine', false, 1, 1, 1, 'model', 'runtime')",
            [],
        )?;
        conn.execute(
            "INSERT INTO ocr_page_metrics(run_id, file_hash, page_no, engine_id, profile_id, model_id,
             runtime_id, runtime_platform, accelerator, status, token_count, avg_tps, elapsed_ms, started_at)
             VALUES ('legacy-run', 'file-a', 1, 'engine', 'profile', 'model', 'runtime', '', '', 'completed', 1, 1.0, 1, 'now')",
            [],
        )?;
        conn.execute(
            "INSERT INTO document_regions(region_id, annotation_id, source_region_key, file_hash,
             page_no, engine_id, profile_id, label, x1, y1, x2, y2, source_span_start,
             source_span_end, content_markdown)
             VALUES ('legacy-region', 'legacy-region', 'legacy-source', 'file-a', 1, 'engine',
             'profile', 'region', 1, 2, 3, 4, 0, 5, '#legacy-region text')",
            [],
        )?;
        conn.execute(
            "INSERT INTO document_text_region_links(file_hash, page_no, region_id, annotation_id,
             text_start, text_end)
             VALUES ('file-a', 1, 'legacy-region', 'legacy-region', 0, 5)",
            [],
        )?;
        conn.execute(
            "INSERT INTO document_region_annotations(region_id, file_hash, page_no, content_markdown)
             VALUES ('legacy-region', 'file-a', 1, '#legacy-region text')",
            [],
        )?;
        conn.execute(
            "INSERT INTO annotation_visibility_overrides(file_hash, page_no, region_id, hidden)
             VALUES ('file-a', 1, 'legacy-region', false)",
            [],
        )?;
        Ok(())
    }

    fn seed_legacy_diagnostic_rows(conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO ingest_diagnostic_spans(span_id, run_id, parent_span_id, name, started_at,
             finished_at, attributes, trace_id, file_hash, page_no, pipeline_step, category,
             annotation_engine, status, ended_at, duration_ms, attributes_json)
             VALUES ('legacy-span', 'legacy-run', NULL, 'span', current_timestamp, current_timestamp,
             '{}'::JSON, 'legacy-run', 'file-a', 1, 'ocr', 'pipeline', 'engine', 'ok', 'done', 1, '{}')",
            [],
        )?;
        conn.execute(
            "INSERT INTO ingest_diagnostic_events(event_id, run_id, span_id, level, message,
             attributes, trace_id, file_hash, page_no, timestamp, event_type, name, severity,
             attributes_json)
             VALUES ('legacy-event', 'legacy-run', 'legacy-span', 'info', 'ok', '{}'::JSON,
             'legacy-run', 'file-a', 1, 'done', 'log', 'ok', 'info', '{}')",
            [],
        )?;
        conn.execute(
            "INSERT INTO ingest_work_units(work_unit_id, run_id, file_hash, page_no, status,
             work_key, phase, engine, provider, model, profile, execution_key, metadata_json)
             VALUES ('legacy-work', 'legacy-run', 'file-a', 1, 'completed', 'legacy-work-key',
             'ocr', 'engine', 'local', 'model', 'profile', 'ocr', '{}')",
            [],
        )?;
        conn.execute(
            "INSERT INTO ingest_model_leases(lease_id, run_id, model_id, execution_key, provider,
             model, status, started_at, metadata_json)
             VALUES ('legacy-lease', 'legacy-run', 'model', 'model', 'local', 'model', 'ok', 'now', '{}')",
            [],
        )?;
        Ok(())
    }

    fn seed_legacy_download_and_replay_rows(conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO download_events(event_id, download_id, download_key, owner_kind, owner_id,
             file_id, file_name, target_path, source_url, event_type, status, downloaded_bytes,
             created_at)
             VALUES ('legacy-download-event', 'legacy-download', 'legacy-download', 'model',
             'model', 'model', 'model.gguf', 'models/model.gguf', 'https://example.invalid',
             'started', 'downloading', 0, 'now')",
            [],
        )?;
        conn.execute(
            "INSERT INTO ocr_stream_events(event_id, sequence, event_type, occurred_at, run_id,
             file_hash, page_no, payload_json)
             VALUES ('legacy-realtime', 99, 'ocr.page.completed', 'now', 'legacy-run',
             'file-a', 1, '{\"run_id\":\"legacy-run\",\"region\":\"legacy-region\"}')",
            [],
        )?;
        Ok(())
    }

    fn text_value(conn: &Connection, sql: &str) -> Result<String> {
        Ok(conn.query_row(sql, [], |row| row.get::<_, String>(0))?)
    }

    fn migration_count(conn: &Connection) -> Result<i64> {
        Ok(conn.query_row("SELECT count(*) FROM persistence_id_migrations", [], |row| {
            row.get::<_, i64>(0)
        })?)
    }
}
