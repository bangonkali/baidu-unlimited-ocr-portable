fn insert_engine_config(conn: &Connection, config: &StoredRunEngineConfig) -> Result<()> {
    let parameters = config.parameters.to_string();
    conn.execute(
        "INSERT INTO ingest_run_engine_configs(
            run_engine_id, run_id, ordinal, engine_kind, engine_id, model_id, profile_id,
            runtime_id, parameters_json, status, error, usable_output_count
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(run_engine_id) DO UPDATE SET
            ordinal = excluded.ordinal, engine_kind = excluded.engine_kind,
            engine_id = excluded.engine_id, model_id = excluded.model_id,
            profile_id = excluded.profile_id, runtime_id = excluded.runtime_id,
            parameters_json = excluded.parameters_json, status = excluded.status,
            error = excluded.error, usable_output_count = excluded.usable_output_count",
        params![
            config.run_engine_id.as_str(),
            config.run_id.as_str(),
            i64::from(config.ordinal),
            config.engine_kind.as_str(),
            config.engine_id.as_str(),
            config.model_id.as_deref(),
            config.profile_id.as_deref(),
            config.runtime_id.as_deref(),
            parameters.as_str(),
            config.status.as_str(),
            config.error.as_deref(),
            i64::from(config.usable_output_count)
        ],
    )?;
    Ok(())
}

fn insert_output_elements(
    conn: &Connection,
    output_id: &str,
    config: &StoredRunEngineConfig,
    page: &StoredPage,
) -> Result<()> {
    for (index, box_record) in page.boxes.iter().enumerate() {
        conn.execute(
            "INSERT INTO document_output_elements(
                element_id, output_id, run_id, run_engine_id, file_hash, page_no, ordinal,
                annotation_id, source_region_key, element_kind, category, markdown, bbox_kind,
                x1, y1, x2, y2, metadata_json
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'TOPLEFT_NORMALIZED_0_999',
                ?, ?, ?, ?, ?)",
            params![
                new_persistence_id().as_str(),
                output_id,
                config.run_id.as_str(),
                config.run_engine_id.as_str(),
                page.file_hash.as_str(),
                i64::from(page.page_no),
                i64::try_from(index).unwrap_or(i64::MAX),
                box_record.annotation_id.as_str(),
                box_record.source_region_key.as_str(),
                box_record.label.as_str(),
                box_record.category.as_str(),
                box_record.content_markdown.as_str(),
                box_record.left_percent / 100.0 * 999.0,
                box_record.top_percent / 100.0 * 999.0,
                (box_record.left_percent + box_record.width_percent) / 100.0 * 999.0,
                (box_record.top_percent + box_record.height_percent) / 100.0 * 999.0,
                "{}"
            ],
        )?;
    }
    if page.boxes.is_empty() && !page.cleaned_text.trim().is_empty() {
        conn.execute(
            "INSERT INTO document_output_elements(
                element_id, output_id, run_id, run_engine_id, file_hash, page_no, ordinal,
                annotation_id, source_region_key, element_kind, category, markdown, metadata_json
             )
             VALUES (?, ?, ?, ?, ?, ?, 0, NULL, NULL, 'markdown', 'page', ?, ?)",
            params![
                new_persistence_id().as_str(),
                output_id,
                config.run_id.as_str(),
                config.run_engine_id.as_str(),
                page.file_hash.as_str(),
                i64::from(page.page_no),
                page.cleaned_text.as_str(),
                "{}"
            ],
        )?;
    }
    Ok(())
}

fn insert_output_spans(
    conn: &Connection,
    output_id: &str,
    config: &StoredRunEngineConfig,
    page: &StoredPage,
) -> Result<()> {
    for span in &page.spans {
        conn.execute(
            "INSERT INTO document_output_spans(
                span_id, output_id, run_id, run_engine_id, file_hash, page_no, annotation_id,
                source_region_key, text_start, text_end, category, metadata_json
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'text', ?)",
            params![
                new_persistence_id().as_str(),
                output_id,
                config.run_id.as_str(),
                config.run_engine_id.as_str(),
                page.file_hash.as_str(),
                i64::from(page.page_no),
                span.annotation_id.as_str(),
                span.source_region_key.as_str(),
                u64_to_i64_saturating(span.start),
                u64_to_i64_saturating(span.end),
                "{}"
            ],
        )?;
    }
    Ok(())
}

fn output_spans(conn: &Connection, output_id: &str) -> Result<Vec<TextRegionSpan>> {
    let mut statement = conn.prepare(
        "SELECT coalesce(annotation_id, span_id), coalesce(annotation_id, span_id),
          coalesce(source_region_key, ''), page_no, text_start, text_end
         FROM document_output_spans
         WHERE output_id = ?
         ORDER BY text_start, span_id",
    )?;
    let rows = statement.query_map(params![output_id], |row| {
        Ok(TextRegionSpan {
            annotation_id: row.get(0)?,
            region_id: row.get(1)?,
            source_region_key: row.get(2)?,
            page_no: i64_to_u32(row.get::<_, i64>(3)?),
            start: i64_to_u64(row.get::<_, i64>(4)?),
            end: i64_to_u64(row.get::<_, i64>(5)?),
        })
    })?;
    collect_rows(rows)
}

fn engine_config_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<StoredRunEngineConfig> {
    Ok(StoredRunEngineConfig {
        run_engine_id: row.get(0)?,
        run_id: row.get(1)?,
        ordinal: i64_to_u32(row.get::<_, i64>(2)?),
        engine_kind: row.get(3)?,
        engine_id: row.get(4)?,
        model_id: row.get(5)?,
        profile_id: row.get(6)?,
        runtime_id: row.get(7)?,
        parameters: json_value(row.get::<_, String>(8)?.as_str()),
        status: row.get(9)?,
        error: row.get(10)?,
        usable_output_count: i64_to_u32(row.get::<_, i64>(11)?),
    })
}

fn preview_result_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<StoredPreviewResult> {
    Ok(StoredPreviewResult {
        run_engine_id: row.get(0)?,
        run_id: row.get(1)?,
        ordinal: i64_to_u32(row.get::<_, i64>(2)?),
        engine_kind: row.get(3)?,
        engine_id: row.get(4)?,
        model_id: row.get(5)?,
        profile_id: row.get(6)?,
        runtime_id: row.get(7)?,
        status: row.get(8)?,
        error: row.get(9)?,
        output_count: i64_to_u32(row.get::<_, i64>(10)?),
        page_count: i64_to_u32(row.get::<_, i64>(11)?),
    })
}
