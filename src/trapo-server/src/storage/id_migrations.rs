impl Repository {
    pub(super) fn migrate_generated_ids_to_uuid_v7(conn: &Connection) -> Result<()> {
        Self::migrate_run_ids(conn)?;
        Self::migrate_region_ids(conn)?;
        Self::migrate_diagnostic_ids(conn)?;
        Self::migrate_work_and_lease_ids(conn)?;
        Self::migrate_download_ids(conn)?;
        Self::migrate_realtime_event_ids(conn)?;
        Self::backfill_annotation_identities(conn)?;
        Ok(())
    }

    fn migrate_run_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "run_id",
            &Self::distinct_text_values(
                conn,
                "SELECT run_id FROM ingest_runs
                 UNION SELECT run_id FROM ingest_run_documents
                 UNION SELECT run_id FROM ingest_work_units
                 UNION SELECT run_id FROM ingest_diagnostic_spans WHERE run_id IS NOT NULL
                 UNION SELECT trace_id FROM ingest_diagnostic_spans WHERE trace_id IS NOT NULL
                 UNION SELECT run_id FROM ingest_diagnostic_events WHERE run_id IS NOT NULL
                 UNION SELECT trace_id FROM ingest_diagnostic_events WHERE trace_id IS NOT NULL
                 UNION SELECT run_id FROM ingest_model_leases
                 UNION SELECT run_id FROM ocr_page_metrics
                 UNION SELECT run_id FROM ocr_stream_events WHERE run_id IS NOT NULL",
            )?,
            &[
                "UPDATE ingest_runs SET run_id = ? WHERE run_id = ?",
                "UPDATE ingest_run_documents SET run_id = ? WHERE run_id = ?",
                "UPDATE ingest_work_units SET run_id = ? WHERE run_id = ?",
                "UPDATE ingest_diagnostic_spans SET run_id = ? WHERE run_id = ?",
                "UPDATE ingest_diagnostic_spans SET trace_id = ? WHERE trace_id = ?",
                "UPDATE ingest_diagnostic_events SET run_id = ? WHERE run_id = ?",
                "UPDATE ingest_diagnostic_events SET trace_id = ? WHERE trace_id = ?",
                "UPDATE ingest_model_leases SET run_id = ? WHERE run_id = ?",
                "UPDATE ocr_page_metrics SET run_id = ? WHERE run_id = ?",
                "UPDATE ocr_stream_events SET run_id = ? WHERE run_id = ?",
            ],
        )
    }

    fn migrate_region_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "region_id",
            &Self::distinct_text_values(
                conn,
                "SELECT region_id FROM document_regions
                 UNION SELECT annotation_id FROM document_regions WHERE annotation_id IS NOT NULL
                 UNION SELECT region_id FROM document_text_region_links
                 UNION SELECT annotation_id FROM document_text_region_links WHERE annotation_id IS NOT NULL
                 UNION SELECT region_id FROM document_region_annotations
                 UNION SELECT region_id FROM annotation_visibility_overrides",
            )?,
            &[
                "UPDATE document_regions SET region_id = ? WHERE region_id = ?",
                "UPDATE document_regions SET annotation_id = ? WHERE annotation_id = ?",
                "UPDATE document_text_region_links SET region_id = ? WHERE region_id = ?",
                "UPDATE document_text_region_links SET annotation_id = ? WHERE annotation_id = ?",
                "UPDATE document_region_annotations SET region_id = ? WHERE region_id = ?",
                "UPDATE annotation_visibility_overrides SET region_id = ? WHERE region_id = ?",
            ],
        )
    }

    fn migrate_diagnostic_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "diagnostic_span_id",
            &Self::distinct_text_values(
                conn,
                "SELECT span_id FROM ingest_diagnostic_spans
                 UNION SELECT parent_span_id FROM ingest_diagnostic_spans WHERE parent_span_id IS NOT NULL
                 UNION SELECT span_id FROM ingest_diagnostic_events WHERE span_id IS NOT NULL",
            )?,
            &[
                "UPDATE ingest_diagnostic_spans SET span_id = ? WHERE span_id = ?",
                "UPDATE ingest_diagnostic_spans SET parent_span_id = ? WHERE parent_span_id = ?",
                "UPDATE ingest_diagnostic_events SET span_id = ? WHERE span_id = ?",
            ],
        )?;
        Self::migrate_id_group(
            conn,
            "diagnostic_event_id",
            &Self::distinct_text_values(conn, "SELECT event_id FROM ingest_diagnostic_events")?,
            &["UPDATE ingest_diagnostic_events SET event_id = ? WHERE event_id = ?"],
        )
    }

    fn migrate_work_and_lease_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "work_unit_id",
            &Self::distinct_text_values(conn, "SELECT work_unit_id FROM ingest_work_units")?,
            &["UPDATE ingest_work_units SET work_unit_id = ? WHERE work_unit_id = ?"],
        )?;
        Self::migrate_id_group(
            conn,
            "model_lease_id",
            &Self::distinct_text_values(conn, "SELECT lease_id FROM ingest_model_leases")?,
            &["UPDATE ingest_model_leases SET lease_id = ? WHERE lease_id = ?"],
        )
    }

    fn migrate_download_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "download_id",
            &Self::distinct_text_values(conn, "SELECT download_id FROM download_events")?,
            &["UPDATE download_events SET download_id = ? WHERE download_id = ?"],
        )?;
        Self::migrate_id_group(
            conn,
            "download_event_id",
            &Self::distinct_text_values(conn, "SELECT event_id FROM download_events")?,
            &["UPDATE download_events SET event_id = ? WHERE event_id = ?"],
        )
    }

    fn migrate_realtime_event_ids(conn: &Connection) -> Result<()> {
        Self::migrate_id_group(
            conn,
            "realtime_event_id",
            &Self::distinct_text_values(conn, "SELECT event_id FROM ocr_stream_events")?,
            &["UPDATE ocr_stream_events SET event_id = ? WHERE event_id = ?"],
        )
    }

    fn distinct_text_values(conn: &Connection, sql: &str) -> Result<Vec<String>> {
        let mut statement = conn.prepare(sql)?;
        let rows = statement.query_map([], |row| row.get::<_, Option<String>>(0))?;
        let mut values = Vec::new();
        for row in rows {
            if let Some(value) = row?
                && !value.is_empty()
            {
                values.push(value);
            }
        }
        values.sort();
        values.dedup();
        Ok(values)
    }

    fn migrate_id_group(
        conn: &Connection,
        id_kind: &str,
        values: &[String],
        update_sql: &[&str],
    ) -> Result<()> {
        for old_id in values {
            if is_uuid_v7(old_id) {
                continue;
            }
            let new_id = Self::mapped_uuid_v7(conn, id_kind, old_id)?;
            for sql in update_sql {
                conn.execute(sql, params![new_id.as_str(), old_id.as_str()])?;
            }
            Self::replace_legacy_id_in_payloads(conn, old_id, &new_id)?;
        }
        Ok(())
    }

    fn mapped_uuid_v7(conn: &Connection, id_kind: &str, old_id: &str) -> Result<String> {
        let existing: Option<String> = conn
            .query_row(
                "SELECT new_id FROM persistence_id_migrations WHERE id_kind = ? AND old_id = ?",
                params![id_kind, old_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            return Ok(existing);
        }
        let new_id = new_persistence_id();
        conn.execute(
            "INSERT INTO persistence_id_migrations(id_kind, old_id, new_id)
             VALUES (?, ?, ?)
             ON CONFLICT(id_kind, old_id) DO NOTHING",
            params![id_kind, old_id, new_id],
        )?;
        Ok(new_id)
    }

    fn replace_legacy_id_in_payloads(conn: &Connection, old_id: &str, new_id: &str) -> Result<()> {
        let pattern = format!("%{old_id}%");
        conn.execute(
            "UPDATE ocr_stream_events SET payload_json = replace(payload_json, ?, ?)
             WHERE payload_json LIKE ?",
            params![old_id, new_id, pattern],
        )?;
        conn.execute(
            "UPDATE document_regions SET content_markdown = replace(content_markdown, ?, ?)
             WHERE content_markdown IS NOT NULL AND content_markdown LIKE ?",
            params![old_id, new_id, pattern],
        )?;
        conn.execute(
            "UPDATE document_region_annotations SET content_markdown = replace(content_markdown, ?, ?)
             WHERE content_markdown LIKE ?",
            params![old_id, new_id, pattern],
        )?;
        Ok(())
    }

    fn backfill_annotation_identities(conn: &Connection) -> Result<()> {
        let mut statement = conn.prepare(
            "SELECT coalesce(annotation_id, region_id), file_hash, page_no, engine_id, profile_id,
                coalesce(source_region_key, region_id), label, x1, y1, x2, y2
             FROM document_regions r
             WHERE NOT EXISTS (
                SELECT 1 FROM document_annotation_identities a
                WHERE a.annotation_id = coalesce(r.annotation_id, r.region_id)
             )
             ORDER BY file_hash, page_no, source_span_start, region_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, f64>(7)?,
                row.get::<_, f64>(8)?,
                row.get::<_, f64>(9)?,
                row.get::<_, f64>(10)?,
            ))
        })?;
        let rows = collect_rows(rows)?;
        drop(statement);
        Self::insert_annotation_identity_backfills(conn, rows)
    }

    fn insert_annotation_identity_backfills(
        conn: &Connection,
        rows: Vec<AnnotationBackfillRow>,
    ) -> Result<()> {
        let mut index = 0_i64;
        for row in rows {
            let (annotation_id, file_hash, page_no, engine_id, profile_id, source_key, label, x1, y1, x2, y2) =
                row;
            let run_id = Self::run_id_for_legacy_annotation(conn, &file_hash, page_no)?;
            conn.execute(
                "INSERT INTO document_annotation_identities(
                    annotation_id, run_id, file_hash, page_no, engine_id, profile_id,
                    source_region_key, discovery_index, label, x1, y1, x2, y2, created_at
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, now())
                 ON CONFLICT(annotation_id) DO NOTHING",
                params![
                    annotation_id,
                    run_id,
                    file_hash,
                    page_no,
                    engine_id,
                    profile_id,
                    source_key,
                    index,
                    label,
                    x1,
                    y1,
                    x2,
                    y2
                ],
            )?;
            index = index.saturating_add(1);
        }
        Ok(())
    }

    fn run_id_for_legacy_annotation(
        conn: &Connection,
        file_hash: &str,
        page_no: i64,
    ) -> Result<String> {
        let run_id: Option<String> = conn
            .query_row(
                "SELECT run_id FROM ocr_page_metrics WHERE file_hash = ? AND page_no = ? LIMIT 1",
                params![file_hash, page_no],
                |row| row.get(0),
            )
            .optional()?;
        Ok(run_id.unwrap_or_else(new_persistence_id))
    }
}

type AnnotationBackfillRow = (String, String, i64, String, String, String, String, f64, f64, f64, f64);
