pub(super) const DIAGNOSTIC_TRACE_RENDERING: &str = r"
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS task_id TEXT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS work_unit_id TEXT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS span_kind TEXT DEFAULT 'operation';
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS activity_kind TEXT DEFAULT 'internal';
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS status_code TEXT DEFAULT 'unset';
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS status_message TEXT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS started_at_ms BIGINT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS ended_at_ms BIGINT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS resource_json TEXT DEFAULT '{}';
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS links_json TEXT DEFAULT '[]';
ALTER TABLE ingest_diagnostic_events ADD COLUMN IF NOT EXISTS timestamp_ms BIGINT;
UPDATE ingest_diagnostic_spans
SET span_kind = coalesce(nullif(span_kind, ''), nullif(category, ''), 'operation')
WHERE span_kind IS NULL OR span_kind = '';
UPDATE ingest_diagnostic_spans
SET activity_kind = coalesce(nullif(activity_kind, ''), 'internal')
WHERE activity_kind IS NULL OR activity_kind = '';
UPDATE ingest_diagnostic_spans
SET status_code = CASE
    WHEN coalesce(error_message, '') <> '' OR coalesce(status, '') IN ('error', 'failed') THEN 'error'
    WHEN coalesce(status, '') IN ('ok', 'completed', 'completed_with_errors') THEN 'ok'
    ELSE coalesce(nullif(status_code, ''), 'unset')
  END
WHERE status_code IS NULL OR status_code = '';
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_trace_parent
  ON ingest_diagnostic_spans(trace_id, parent_span_id, started_at);
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_trace_ms
  ON ingest_diagnostic_spans(trace_id, parent_span_id, started_at_ms);
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_task
  ON ingest_diagnostic_spans(task_id, started_at);
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_work_unit
  ON ingest_diagnostic_spans(work_unit_id, started_at);
";
