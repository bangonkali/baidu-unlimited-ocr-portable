impl Repository {
    pub(crate) async fn upsert_work_unit(&self, unit: &WorkUnitUpsert) -> Result<()> {
        let unit = unit.clone();
        let metadata = unit.metadata.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ingest_work_units(
                    work_unit_id, run_id, file_hash, page_no, status, queued_at, work_key, phase,
                    engine, provider, model, profile, execution_key, artifact_variant, metadata_json
                 )
                 VALUES (?, ?, ?, ?, 'planned', current_timestamp, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(run_id, work_key) DO UPDATE SET
                    file_hash = excluded.file_hash, page_no = excluded.page_no,
                    work_key = excluded.work_key, phase = excluded.phase, engine = excluded.engine,
                    provider = excluded.provider, model = excluded.model, profile = excluded.profile,
                    execution_key = excluded.execution_key, artifact_variant = excluded.artifact_variant,
                    metadata_json = excluded.metadata_json,
                    status = CASE
                      WHEN ingest_work_units.status IN ('ok','error','skipped') THEN ingest_work_units.status
                      ELSE ingest_work_units.status
                    END",
                params![
                    unit.work_unit_id.as_str(),
                    unit.run_id.as_str(),
                    unit.file_hash.as_deref(),
                    unit.page_no.map_or(0, i64::from),
                    unit.work_key.as_str(),
                    unit.phase.as_str(),
                    unit.engine.as_str(),
                    unit.provider.as_str(),
                    unit.model.as_str(),
                    unit.profile.as_deref(),
                    unit.execution_key.as_str(),
                    unit.artifact_variant.as_deref(),
                    metadata.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn start_work_unit(&self, run_id: &str, work_key: &str) -> Result<()> {
        let run_id = run_id.to_string();
        let work_key = work_key.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "UPDATE ingest_work_units
                 SET status = 'running', attempts = attempts + 1, attempt_count = attempt_count + 1,
                     started_at = current_timestamp, finished_at = NULL, duration_ms = NULL, error = NULL
                 WHERE run_id = ? AND work_key = ?",
                params![run_id.as_str(), work_key.as_str()],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn finish_work_unit(
        &self,
        run_id: &str,
        work_key: &str,
        status: &str,
        result: &Value,
        error: Option<&str>,
    ) -> Result<()> {
        let run_id = run_id.to_string();
        let work_key = work_key.to_string();
        let status = status.to_string();
        let result = result.to_string();
        let error = error.map(str::to_string);
        self.with_write(move |conn| {
            conn.execute(
                "UPDATE ingest_work_units
                 SET status = ?, started_at = coalesce(started_at, queued_at, current_timestamp),
                     finished_at = current_timestamp,
                     duration_ms = date_diff(
                       'millisecond',
                       coalesce(started_at, queued_at, current_timestamp),
                       current_timestamp
                     ),
                     result_json = ?, error = ?
                 WHERE run_id = ? AND work_key = ?",
                params![
                    status.as_str(),
                    result.as_str(),
                    error.as_deref(),
                    run_id.as_str(),
                    work_key.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn insert_diagnostic_span(&self, span: &DiagnosticSpanInsert) -> Result<()> {
        let span = span.clone();
        let attributes = span.attributes.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ingest_diagnostic_spans(
                    span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
                    trace_id, file_hash, page_no, pipeline_step, category, annotation_engine, status,
                    ended_at, duration_ms, attributes_json, error_type, error_message, error_stack,
                    task_id, work_unit_id, span_kind
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?::JSON, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(span_id) DO UPDATE SET
                    finished_at = excluded.finished_at, attributes = excluded.attributes,
                    trace_id = excluded.trace_id, file_hash = excluded.file_hash, page_no = excluded.page_no,
                    pipeline_step = excluded.pipeline_step, category = excluded.category,
                    annotation_engine = excluded.annotation_engine, status = excluded.status,
                    ended_at = excluded.ended_at, duration_ms = excluded.duration_ms,
                    attributes_json = excluded.attributes_json, error_type = excluded.error_type,
                    error_message = excluded.error_message, error_stack = excluded.error_stack,
                    task_id = excluded.task_id, work_unit_id = excluded.work_unit_id,
                    span_kind = excluded.span_kind",
                params![
                    span.span_id.as_str(),
                    span.run_id.as_deref(),
                    span.parent_span_id.as_deref(),
                    span.name.as_str(),
                    span.started_at.as_str(),
                    span.ended_at.as_str(),
                    attributes.as_str(),
                    span.trace_id.as_str(),
                    span.file_hash.as_deref(),
                    span.page_no.map(i64::from),
                    span.pipeline_step.as_str(),
                    span.category.as_str(),
                    span.annotation_engine.as_deref(),
                    span.status.as_str(),
                    span.ended_at.as_str(),
                    span.duration_ms,
                    attributes.as_str(),
                    span.error_type.as_deref(),
                    span.error_message.as_deref(),
                    span.error_stack.as_deref(),
                    span.task_id.as_deref(),
                    span.work_unit_id.as_deref(),
                    span.span_kind.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn insert_diagnostic_event(&self, event: &DiagnosticEventInsert) -> Result<()> {
        let event = event.clone();
        let attributes = event.attributes.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ingest_diagnostic_events(
                    event_id, run_id, span_id, level, message, attributes, created_at,
                    trace_id, file_hash, page_no, timestamp, event_type, name, severity, attributes_json
                 )
                 VALUES (?, ?, ?, ?, ?, ?::JSON, current_timestamp, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(event_id) DO NOTHING",
                params![
                    event.event_id.as_str(),
                    event.run_id.as_deref(),
                    event.span_id.as_deref(),
                    event.severity.as_str(),
                    event.message.as_str(),
                    attributes.as_str(),
                    event.trace_id.as_str(),
                    event.file_hash.as_deref(),
                    event.page_no.map(i64::from),
                    event.timestamp.as_str(),
                    event.event_type.as_str(),
                    event.name.as_str(),
                    event.severity.as_str(),
                    attributes.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn insert_model_lease(
        &self,
        lease: &DiagnosticModelLeaseInsert,
    ) -> Result<()> {
        let lease = lease.clone();
        let lease_id = new_persistence_id();
        let started_at = Utc::now().to_rfc3339();
        let metadata = lease.metadata.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ingest_model_leases(
                    lease_id, run_id, model_id, acquired_at, execution_key, provider, model,
                    status, started_at, metadata_json
                 )
                 VALUES (?, ?, ?, current_timestamp, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(run_id, execution_key) DO UPDATE SET
                    status = excluded.status, metadata_json = excluded.metadata_json",
                params![
                    lease_id.as_str(),
                    lease.run_id.as_str(),
                    lease.model.as_str(),
                    lease.execution_key.as_str(),
                    lease.provider.as_str(),
                    lease.model.as_str(),
                    lease.status.as_str(),
                    started_at.as_str(),
                    metadata.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }
}
