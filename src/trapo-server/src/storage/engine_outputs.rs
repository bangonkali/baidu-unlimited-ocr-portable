impl Repository {
    pub(crate) async fn replace_run_engine_configs(
        &self,
        run_id: &str,
        configs: &[StoredRunEngineConfig],
    ) -> Result<()> {
        let run_id = run_id.to_string();
        let configs = configs.to_vec();
        self.with_write(move |conn| {
            conn.execute(
                "DELETE FROM ingest_run_engine_configs WHERE run_id = ?",
                params![run_id.as_str()],
            )?;
            for config in configs {
                insert_engine_config(&conn, &config)?;
            }
            Ok(())
        })
        .await
    }

    pub(crate) async fn update_run_engine_config_status(
        &self,
        run_engine_id: &str,
        status: &str,
        error: Option<&str>,
        usable_output_count: u32,
    ) -> Result<()> {
        let run_engine_id = run_engine_id.to_string();
        let status = status.to_string();
        let error = error.map(str::to_string);
        self.with_write(move |conn| {
            conn.execute(
                "UPDATE ingest_run_engine_configs
                 SET status = ?,
                     started_at = CASE WHEN ? = 'running' THEN coalesce(started_at, CAST(now() AS VARCHAR)) ELSE started_at END,
                     finished_at = CASE WHEN ? IN ('completed','completed_with_errors','failed','cancelled') THEN CAST(now() AS VARCHAR) ELSE finished_at END,
                     duration_ms = CASE
                       WHEN ? IN ('completed','completed_with_errors','failed','cancelled')
                         AND started_at IS NOT NULL
                       THEN date_diff('millisecond', CAST(started_at AS TIMESTAMP), now())
                       ELSE duration_ms
                     END,
                     error = ?,
                     usable_output_count = ?
                 WHERE run_engine_id = ?",
                params![
                    status.as_str(),
                    status.as_str(),
                    status.as_str(),
                    status.as_str(),
                    error.as_deref(),
                    i64::from(usable_output_count),
                    run_engine_id.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn replace_page_output(
        &self,
        config: &StoredRunEngineConfig,
        page: &StoredPage,
        work_unit_id: Option<&str>,
        output_kind: &str,
        elapsed_ms: u64,
    ) -> Result<()> {
        let config = config.clone();
        let page = page.clone();
        let work_unit_id = work_unit_id.map(str::to_string);
        let output_kind = output_kind.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "DELETE FROM document_output_spans
                 WHERE run_engine_id = ? AND file_hash = ? AND page_no = ?",
                params![
                    config.run_engine_id.as_str(),
                    page.file_hash.as_str(),
                    i64::from(page.page_no)
                ],
            )?;
            conn.execute(
                "DELETE FROM document_output_elements
                 WHERE run_engine_id = ? AND file_hash = ? AND page_no = ?",
                params![
                    config.run_engine_id.as_str(),
                    page.file_hash.as_str(),
                    i64::from(page.page_no)
                ],
            )?;
            conn.execute(
                "DELETE FROM document_page_outputs
                 WHERE run_engine_id = ? AND file_hash = ? AND page_no = ?",
                params![
                    config.run_engine_id.as_str(),
                    page.file_hash.as_str(),
                    i64::from(page.page_no)
                ],
            )?;
            let output_id = new_persistence_id();
            let metadata = page_output_provenance(&config).to_string();
            conn.execute(
                "INSERT INTO document_page_outputs(
                    output_id, run_id, run_engine_id, work_unit_id, file_hash, page_no,
                    output_kind, engine_id, engine_kind, model_id, profile_id, runtime_id,
                    status, markdown, raw_text, error, elapsed_ms, metadata_json
                 )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    output_id.as_str(),
                    config.run_id.as_str(),
                    config.run_engine_id.as_str(),
                    work_unit_id.as_deref(),
                    page.file_hash.as_str(),
                    i64::from(page.page_no),
                    output_kind.as_str(),
                    config.engine_id.as_str(),
                    config.engine_kind.as_str(),
                    config.model_id.as_deref(),
                    config.profile_id.as_deref(),
                    config.runtime_id.as_deref(),
                    page.status.as_str(),
                    page.cleaned_text.as_str(),
                    page.raw_text.as_str(),
                    page.error.as_deref(),
                    u64_to_i64_saturating(elapsed_ms),
                    metadata.as_str()
                ],
            )?;
            insert_output_elements(&conn, &output_id, &config, &page)?;
            insert_output_spans(&conn, &output_id, &config, &page)?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn preview_results_for_document(
        &self,
        run_id: &str,
        file_hash: &str,
    ) -> Result<Vec<StoredPreviewResult>> {
        let run_id = run_id.to_string();
        let file_hash = file_hash.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT c.run_engine_id, c.run_id, c.ordinal, c.engine_kind, c.engine_id,
                  c.model_id, c.profile_id, c.runtime_id, c.status, c.error,
                  count(o.output_id) AS output_count,
                  count(DISTINCT o.page_no) AS page_count,
                  max(o.metadata_json) AS provenance_json
                 FROM ingest_run_engine_configs c
                 JOIN document_page_outputs o
                   ON o.run_engine_id = c.run_engine_id
                  AND o.file_hash = ?
                  AND o.status = 'completed'
                 WHERE c.run_id = ?
                   AND c.status IN ('completed','completed_with_errors')
                 GROUP BY c.run_engine_id, c.run_id, c.ordinal, c.engine_kind, c.engine_id,
                   c.model_id, c.profile_id, c.runtime_id, c.status, c.error
                 HAVING count(o.output_id) > 0
                 ORDER BY c.ordinal",
            )?;
            let rows = statement.query_map(
                params![file_hash.as_str(), run_id.as_str()],
                preview_result_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }

    pub(crate) async fn load_document_regions_for_run_engine(
        &self,
        file_hash: &str,
        run_engine_id: &str,
    ) -> Result<Vec<OverlayBox>> {
        let file_hash = file_hash.to_string();
        let run_engine_id = run_engine_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT coalesce(annotation_id, element_id), coalesce(annotation_id, element_id),
                  coalesce(source_region_key, ''), element_kind, category, markdown, page_no,
                  coalesce(x1, 0), coalesce(y1, 0), coalesce(x2, 0), coalesce(y2, 0),
                  coalesce(bbox_kind, 'axis_aligned'), coalesce(metadata_json, '{}')
                 FROM document_output_elements
                 WHERE run_engine_id = ? AND file_hash = ? AND x1 IS NOT NULL
                 ORDER BY page_no, ordinal",
            )?;
            let rows = statement.query_map(
                params![run_engine_id.as_str(), file_hash.as_str()],
                |row| {
                    let x1 = row.get::<_, f64>(7)?;
                    let y1 = row.get::<_, f64>(8)?;
                    let x2 = row.get::<_, f64>(9)?;
                    let y2 = row.get::<_, f64>(10)?;
                    let left_percent = normalized_to_percent(x1);
                    let top_percent = normalized_to_percent(y1);
                    let width_percent = normalized_to_percent(x2 - x1);
                    let height_percent = normalized_to_percent(y2 - y1);
                    let bbox_kind = row.get::<_, String>(11)?;
                    let metadata_json = row.get::<_, String>(12)?;
                    Ok(OverlayBox {
                        annotation_id: row.get(0)?,
                        region_id: row.get(1)?,
                        source_region_key: row.get(2)?,
                        label: row.get(3)?,
                        category: row.get(4)?,
                        content_markdown: row.get(5)?,
                        content_html: None,
                        page_no: i64_to_u32(row.get::<_, i64>(6)?),
                        left_percent,
                        top_percent,
                        width_percent,
                        height_percent,
                        hidden: false,
                        geometry: Some(output_element_geometry(
                            &metadata_json,
                            &bbox_kind,
                            left_percent,
                            top_percent,
                            width_percent,
                            height_percent,
                        )),
                    })
                },
            )?;
            collect_rows(rows)
        })
        .await
    }

    pub(crate) async fn load_document_text_for_run_engine(
        &self,
        file_hash: &str,
        run_engine_id: &str,
    ) -> Result<Vec<PageTextRecord>> {
        let file_hash = file_hash.to_string();
        let run_engine_id = run_engine_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT output_id, page_no, markdown
                 FROM document_page_outputs
                 WHERE run_engine_id = ? AND file_hash = ?
                 ORDER BY page_no",
            )?;
            let rows = statement.query_map(
                params![run_engine_id.as_str(), file_hash.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        i64_to_u32(row.get::<_, i64>(1)?),
                        row.get::<_, String>(2)?,
                    ))
                },
            )?;
            let outputs = collect_rows(rows)?;
            let mut pages = Vec::with_capacity(outputs.len());
            for (output_id, page_no, text) in outputs {
                pages.push(PageTextRecord {
                    page_no,
                    text,
                    spans: output_spans(&conn, &output_id)?,
                });
            }
            Ok(pages)
        })
        .await
    }
}

fn output_element_geometry(
    metadata_json: &str,
    bbox_kind: &str,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
) -> OcrGeometry {
    let geometry_json = json_value(metadata_json)
        .get("geometry")
        .map_or_else(|| "{}".to_string(), serde_json::Value::to_string);
    OcrGeometry::from_storage_json(
        &geometry_json,
        bbox_kind,
        crate::workbench_types::OcrGeometryBounds {
            left,
            top,
            width,
            height,
        },
        None,
    )
}
