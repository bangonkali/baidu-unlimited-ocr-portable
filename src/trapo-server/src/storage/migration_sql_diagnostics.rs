pub(super) const DIAGNOSTIC_TRACE_RENDERING: &str = r"
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS task_id TEXT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS work_unit_id TEXT;
ALTER TABLE ingest_diagnostic_spans ADD COLUMN IF NOT EXISTS span_kind TEXT DEFAULT 'operation';
UPDATE ingest_diagnostic_spans
SET span_kind = coalesce(nullif(span_kind, ''), nullif(category, ''), 'operation')
WHERE span_kind IS NULL OR span_kind = '';
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_trace_parent
  ON ingest_diagnostic_spans(trace_id, parent_span_id, started_at);
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_task
  ON ingest_diagnostic_spans(task_id, started_at);
CREATE INDEX IF NOT EXISTS idx_ingest_diag_spans_work_unit
  ON ingest_diagnostic_spans(work_unit_id, started_at);
";
