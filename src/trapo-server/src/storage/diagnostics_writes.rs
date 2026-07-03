impl Repository {
    pub fn upsert_work_unit(&self, unit: &WorkUnitUpsert) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO ingest_work_units(
                work_unit_id, run_id, file_hash, page_no, status, queued_at, work_key, phase,
                engine, provider, model, profile, execution_key, artifact_variant, metadata_json
             )
             VALUES (?, ?, ?, ?, 'planned', current_timestamp, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(work_unit_id) DO UPDATE SET
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
                unit.work_unit_id,
                unit.run_id,
                unit.file_hash,
                unit.page_no.map(i64::from).unwrap_or(0),
                unit.work_key,
                unit.phase,
                unit.engine,
                unit.provider,
                unit.model,
                unit.profile,
                unit.execution_key,
                unit.artifact_variant,
                unit.metadata.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn start_work_unit(&self, run_id: &str, work_key: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE ingest_work_units
             SET status = 'running', attempts = attempts + 1, attempt_count = attempt_count + 1,
                 started_at = current_timestamp, finished_at = NULL, duration_ms = NULL, error = NULL
             WHERE run_id = ? AND work_key = ?",
            params![run_id, work_key],
        )?;
        Ok(())
    }

    pub fn finish_work_unit(
        &self,
        run_id: &str,
        work_key: &str,
        status: &str,
        result: &Value,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "UPDATE ingest_work_units
             SET status = ?, finished_at = current_timestamp,
                 duration_ms = CASE
                   WHEN started_at IS NULL THEN NULL
                   ELSE date_diff('millisecond', started_at, current_timestamp)
                 END,
                 result_json = ?, error = ?
             WHERE run_id = ? AND work_key = ?",
            params![status, result.to_string(), error, run_id, work_key],
        )?;
        Ok(())
    }

    pub fn insert_diagnostic_span(&self, span: &DiagnosticSpanInsert) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO ingest_diagnostic_spans(
                span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
                trace_id, file_hash, page_no, pipeline_step, category, annotation_engine, status,
                ended_at, duration_ms, attributes_json, error_type, error_message, error_stack
             )
             VALUES (?, ?, ?, ?, ?, ?, ?::JSON, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(span_id) DO UPDATE SET
                finished_at = excluded.finished_at, attributes = excluded.attributes,
                trace_id = excluded.trace_id, file_hash = excluded.file_hash, page_no = excluded.page_no,
                pipeline_step = excluded.pipeline_step, category = excluded.category,
                annotation_engine = excluded.annotation_engine, status = excluded.status,
                ended_at = excluded.ended_at, duration_ms = excluded.duration_ms,
                attributes_json = excluded.attributes_json, error_type = excluded.error_type,
                error_message = excluded.error_message, error_stack = excluded.error_stack",
            params![
                span.span_id,
                span.run_id,
                span.parent_span_id,
                span.name,
                span.started_at,
                span.ended_at,
                span.attributes.to_string(),
                span.trace_id,
                span.file_hash,
                span.page_no.map(i64::from),
                span.pipeline_step,
                span.category,
                span.annotation_engine,
                span.status,
                span.ended_at,
                span.duration_ms,
                span.attributes.to_string(),
                span.error_type,
                span.error_message,
                span.error_stack
            ],
        )?;
        Ok(())
    }

    pub fn insert_diagnostic_event(&self, event: &DiagnosticEventInsert) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO ingest_diagnostic_events(
                event_id, run_id, span_id, level, message, attributes, created_at,
                trace_id, file_hash, page_no, timestamp, event_type, name, severity, attributes_json
             )
             VALUES (?, ?, ?, ?, ?, ?::JSON, current_timestamp, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(event_id) DO NOTHING",
            params![
                event.event_id,
                event.run_id,
                event.span_id,
                event.severity,
                event.message,
                event.attributes.to_string(),
                event.trace_id,
                event.file_hash,
                event.page_no.map(i64::from),
                event.timestamp,
                event.event_type,
                event.name,
                event.severity,
                event.attributes.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn insert_model_lease(
        &self,
        run_id: &str,
        execution_key: &str,
        provider: &str,
        model: &str,
        status: &str,
        metadata: &Value,
    ) -> Result<()> {
        let conn = self.connect()?;
        let lease_id = format!("{run_id}:{execution_key}");
        conn.execute(
            "INSERT INTO ingest_model_leases(
                lease_id, run_id, model_id, acquired_at, execution_key, provider, model,
                status, started_at, metadata_json
             )
             VALUES (?, ?, ?, current_timestamp, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(lease_id) DO UPDATE SET
                status = excluded.status, metadata_json = excluded.metadata_json",
            params![
                lease_id,
                run_id,
                model,
                execution_key,
                provider,
                model,
                status,
                Utc::now().to_rfc3339(),
                metadata.to_string()
            ],
        )?;
        Ok(())
    }
}
