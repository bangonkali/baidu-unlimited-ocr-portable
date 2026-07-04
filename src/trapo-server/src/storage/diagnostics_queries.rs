impl Repository {
    pub async fn diagnostic_runs(&self, limit: u32) -> Result<Vec<DiagnosticRunRow>> {
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT r.run_id, r.root_path, r.status, r.started_at::VARCHAR, r.finished_at::VARCHAR,
                    coalesce(date_diff('millisecond', r.started_at, coalesce(r.finished_at, current_timestamp)), 0),
                    count(s.span_id), count(CASE WHEN s.status = 'error' THEN 1 END),
                    count(DISTINCT s.file_hash) FILTER (WHERE s.file_hash IS NOT NULL),
                    count(DISTINCT s.file_hash || ':' || CAST(s.page_no AS VARCHAR))
                      FILTER (WHERE s.file_hash IS NOT NULL AND s.page_no IS NOT NULL)
                 FROM ingest_runs r
                 LEFT JOIN ingest_diagnostic_spans s ON s.run_id = r.run_id
                 GROUP BY r.run_id, r.root_path, r.status, r.started_at, r.finished_at
                 ORDER BY r.started_at DESC
                 LIMIT ?",
            )?;
            let rows = statement.query_map(params![i64::from(limit.clamp(1, 200))], |row| {
                Ok(DiagnosticRunRow {
                    run_id: row.get(0)?,
                    root_path: row.get(1)?,
                    status: row.get(2)?,
                    started_at: row.get(3)?,
                    finished_at: row.get(4)?,
                    duration_ms: row.get::<_, i64>(5)? as f64,
                    span_count: i64_to_u32(row.get::<_, i64>(6)?),
                    error_count: i64_to_u32(row.get::<_, i64>(7)?),
                    file_count: i64_to_u32(row.get::<_, i64>(8)?),
                    page_count: i64_to_u32(row.get::<_, i64>(9)?),
                })
            })?;
            collect_rows(rows)
        })
        .await
    }

    pub async fn diagnostic_trace(
        &self,
        filter: &DiagnosticTraceFilter<'_>,
    ) -> Result<(Vec<DiagnosticSpanRow>, Vec<DiagnosticEventRow>)> {
        let filter = OwnedDiagnosticTraceFilter::from(filter);
        let repository = self.clone();
        self.with_read(move |conn| {
            Ok((
                repository.diagnostic_spans(&conn, &filter)?,
                repository.diagnostic_events(&conn, &filter)?,
            ))
        })
        .await
    }

    pub async fn diagnostic_work_units(
        &self,
        run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<DiagnosticWorkUnitRow>> {
        let run_id = run_id.map(str::to_string);
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT wu.work_unit_id, wu.run_id, coalesce(wu.work_key, wu.work_unit_id), wu.file_hash,
                    f.display_name, l.relative_path, wu.page_no, coalesce(wu.phase, 'ocr'),
                    coalesce(wu.engine, 'unlimited-ocr-ffi'), coalesce(wu.provider, 'local'),
                    coalesce(wu.model, coalesce(r.model_id, '')), wu.profile,
                    coalesce(wu.execution_key, ''), wu.artifact_variant, wu.status,
                    coalesce(wu.attempt_count, wu.attempts), wu.started_at::VARCHAR,
                    wu.finished_at::VARCHAR, wu.duration_ms, wu.error,
                    coalesce(wu.result_json, '{}'), coalesce(wu.metadata_json, '{}')
                 FROM ingest_work_units wu
                 LEFT JOIN ingest_runs r ON r.run_id = wu.run_id
                 LEFT JOIN files f ON f.file_hash = wu.file_hash
                 LEFT JOIN file_locations l ON l.file_hash = wu.file_hash
                 WHERE (? IS NULL OR wu.run_id = ?)
                 QUALIFY row_number() OVER (PARTITION BY wu.work_unit_id ORDER BY l.observed_at DESC NULLS LAST) = 1
                 ORDER BY wu.queued_at ASC, wu.work_unit_id ASC
                 LIMIT ?",
            )?;
            let rows = statement.query_map(
                params![
                    run_id.as_deref(),
                    run_id.as_deref(),
                    i64::from(limit.clamp(1, 100_000))
                ],
                work_unit_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }

    pub async fn diagnostic_model_leases(
        &self,
        run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<DiagnosticModelLeaseRow>> {
        let run_id = run_id.map(str::to_string);
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT lease_id, run_id, coalesce(execution_key, ''), coalesce(provider, 'local'),
                    coalesce(model, model_id), requested_context_tokens, verified_context_tokens,
                    coalesce(status, 'ok'), coalesce(started_at, acquired_at::VARCHAR),
                    finished_at, duration_ms, error, coalesce(metadata_json, '{}')
                 FROM ingest_model_leases
                 WHERE (? IS NULL OR run_id = ?)
                 ORDER BY acquired_at ASC, lease_id ASC
                 LIMIT ?",
            )?;
            let rows = statement.query_map(
                params![
                    run_id.as_deref(),
                    run_id.as_deref(),
                    i64::from(limit.clamp(1, 10_000))
                ],
                lease_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }

    fn diagnostic_spans(
        &self,
        conn: &Connection,
        filter: &OwnedDiagnosticTraceFilter,
    ) -> Result<Vec<DiagnosticSpanRow>> {
        let page_no = filter.page_no.map(i64::from);
        let status = filter.status.as_deref().filter(|value| *value != "all");
        let query = filter.q.as_deref().map(normalized_like);
        let mut statement = conn.prepare(
            "SELECT span_id, coalesce(trace_id, run_id), parent_span_id, run_id, file_hash, page_no,
                name, coalesce(pipeline_step, name), coalesce(category, 'pipeline'),
                annotation_engine, coalesce(status, 'ok'), started_at::VARCHAR,
                coalesce(ended_at, finished_at::VARCHAR, started_at::VARCHAR), coalesce(duration_ms, 0),
                coalesce(attributes_json, '{}'), error_type, error_message, error_stack
             FROM ingest_diagnostic_spans
             WHERE (? IS NULL OR run_id = ?)
               AND (? IS NULL OR file_hash = ?)
               AND (? IS NULL OR page_no = ?)
               AND (? IS NULL OR status = ?)
               AND (? IS NULL OR lower(name || ' ' || coalesce(pipeline_step, '') || ' ' ||
                   coalesce(category, '') || ' ' || coalesce(annotation_engine, '') || ' ' ||
                   coalesce(error_message, '')) LIKE ?)
             ORDER BY started_at ASC, duration_ms DESC
             LIMIT ?",
        )?;
        let rows = statement.query_map(
            params![
                filter.run_id.as_deref(),
                filter.run_id.as_deref(),
                filter.file_hash.as_deref(),
                filter.file_hash.as_deref(),
                page_no,
                page_no,
                status,
                status,
                query.as_deref(),
                query.as_deref(),
                i64::from(filter.limit.clamp(1, 100_000))
            ],
            span_from_row,
        )?;
        collect_rows(rows)
    }

    fn diagnostic_events(
        &self,
        conn: &Connection,
        filter: &OwnedDiagnosticTraceFilter,
    ) -> Result<Vec<DiagnosticEventRow>> {
        let page_no = filter.page_no.map(i64::from);
        let query = filter.q.as_deref().map(normalized_like);
        let mut statement = conn.prepare(
            "SELECT event_id, coalesce(trace_id, run_id), span_id, run_id, file_hash, page_no,
                coalesce(timestamp, created_at::VARCHAR), coalesce(event_type, 'log'),
                coalesce(name, level), coalesce(severity, level), message,
                coalesce(attributes_json, '{}')
             FROM ingest_diagnostic_events
             WHERE (? IS NULL OR run_id = ?)
               AND (? IS NULL OR file_hash = ?)
               AND (? IS NULL OR page_no = ?)
               AND (? IS NULL OR lower(coalesce(name, '') || ' ' || coalesce(message, '')) LIKE ?)
             ORDER BY created_at ASC
             LIMIT ?",
        )?;
        let rows = statement.query_map(
            params![
                filter.run_id.as_deref(),
                filter.run_id.as_deref(),
                filter.file_hash.as_deref(),
                filter.file_hash.as_deref(),
                page_no,
                page_no,
                query.as_deref(),
                query.as_deref(),
                i64::from(filter.limit.clamp(1, 100_000))
            ],
            event_from_row,
        )?;
        collect_rows(rows)
    }
}

pub struct DiagnosticTraceFilter<'a> {
    pub run_id: Option<&'a str>,
    pub file_hash: Option<&'a str>,
    pub page_no: Option<u32>,
    pub status: Option<&'a str>,
    pub q: Option<&'a str>,
    pub limit: u32,
}

struct OwnedDiagnosticTraceFilter {
    run_id: Option<String>,
    file_hash: Option<String>,
    page_no: Option<u32>,
    status: Option<String>,
    q: Option<String>,
    limit: u32,
}

impl From<&DiagnosticTraceFilter<'_>> for OwnedDiagnosticTraceFilter {
    fn from(filter: &DiagnosticTraceFilter<'_>) -> Self {
        Self {
            run_id: filter.run_id.map(str::to_string),
            file_hash: filter.file_hash.map(str::to_string),
            page_no: filter.page_no,
            status: filter.status.map(str::to_string),
            q: filter.q.map(str::to_string),
            limit: filter.limit,
        }
    }
}

fn normalized_like(value: &str) -> String {
    format!("%{}%", value.trim().to_lowercase())
}

fn json_value(value: String) -> Value {
    serde_json::from_str(&value).unwrap_or(Value::Null)
}

fn span_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<DiagnosticSpanRow> {
    Ok(DiagnosticSpanRow {
        span_id: row.get(0)?,
        trace_id: row.get(1)?,
        parent_span_id: row.get(2)?,
        run_id: row.get(3)?,
        file_hash: row.get(4)?,
        page_no: row.get::<_, Option<i64>>(5)?.map(i64_to_u32),
        name: row.get(6)?,
        pipeline_step: row.get(7)?,
        category: row.get(8)?,
        annotation_engine: row.get(9)?,
        status: row.get(10)?,
        started_at: row.get(11)?,
        ended_at: row.get(12)?,
        duration_ms: row.get(13)?,
        attributes: json_value(row.get(14)?),
        error_type: row.get(15)?,
        error_message: row.get(16)?,
        error_stack: row.get(17)?,
    })
}

fn event_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<DiagnosticEventRow> {
    Ok(DiagnosticEventRow {
        event_id: row.get(0)?,
        trace_id: row.get(1)?,
        span_id: row.get(2)?,
        run_id: row.get(3)?,
        file_hash: row.get(4)?,
        page_no: row.get::<_, Option<i64>>(5)?.map(i64_to_u32),
        timestamp: row.get(6)?,
        event_type: row.get(7)?,
        name: row.get(8)?,
        severity: row.get(9)?,
        message: row.get(10)?,
        attributes: json_value(row.get(11)?),
    })
}
