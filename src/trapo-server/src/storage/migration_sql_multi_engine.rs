pub(super) const MULTI_ENGINE_INGEST: &str = r"
CREATE TABLE IF NOT EXISTS ingest_run_engine_configs (
  run_engine_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  engine_kind TEXT NOT NULL,
  engine_id TEXT NOT NULL,
  model_id TEXT,
  profile_id TEXT,
  runtime_id TEXT,
  parameters_json TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'queued',
  queued_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR),
  started_at TEXT,
  finished_at TEXT,
  duration_ms DOUBLE,
  error TEXT,
  usable_output_count INTEGER NOT NULL DEFAULT 0,
  UNIQUE(run_id, ordinal)
);
CREATE INDEX IF NOT EXISTS idx_run_engine_configs_run
  ON ingest_run_engine_configs(run_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_run_engine_configs_status
  ON ingest_run_engine_configs(run_id, status);

ALTER TABLE ingest_work_units ADD COLUMN IF NOT EXISTS run_engine_id TEXT;
CREATE INDEX IF NOT EXISTS idx_ingest_work_units_run_engine
  ON ingest_work_units(run_engine_id, status);

ALTER TABLE document_run_page_ocr ADD COLUMN IF NOT EXISTS run_engine_id TEXT;
CREATE INDEX IF NOT EXISTS idx_document_run_page_ocr_engine
  ON document_run_page_ocr(run_engine_id, file_hash, page_no);

ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS run_engine_id TEXT;
CREATE INDEX IF NOT EXISTS idx_regions_run_engine_file_page
  ON document_regions(run_engine_id, file_hash, page_no);

ALTER TABLE document_text_region_links ADD COLUMN IF NOT EXISTS run_engine_id TEXT;
CREATE INDEX IF NOT EXISTS idx_text_links_run_engine_file_page
  ON document_text_region_links(run_engine_id, file_hash, page_no);

CREATE TABLE IF NOT EXISTS document_page_outputs (
  output_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  run_engine_id TEXT NOT NULL,
  work_unit_id TEXT,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  output_kind TEXT NOT NULL,
  engine_id TEXT NOT NULL,
  engine_kind TEXT NOT NULL,
  model_id TEXT,
  profile_id TEXT,
  runtime_id TEXT,
  status TEXT NOT NULL,
  markdown TEXT NOT NULL DEFAULT '',
  raw_text TEXT NOT NULL DEFAULT '',
  error TEXT,
  elapsed_ms INTEGER,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR),
  updated_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR),
  UNIQUE(run_engine_id, file_hash, page_no)
);
CREATE INDEX IF NOT EXISTS idx_document_page_outputs_run_file
  ON document_page_outputs(run_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_document_page_outputs_engine_file
  ON document_page_outputs(run_engine_id, file_hash, page_no);

CREATE TABLE IF NOT EXISTS document_output_elements (
  element_id TEXT PRIMARY KEY,
  output_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  run_engine_id TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  ordinal INTEGER NOT NULL,
  annotation_id TEXT,
  source_region_key TEXT,
  element_kind TEXT NOT NULL,
  category TEXT NOT NULL,
  markdown TEXT NOT NULL DEFAULT '',
  bbox_kind TEXT,
  x1 DOUBLE,
  y1 DOUBLE,
  x2 DOUBLE,
  y2 DOUBLE,
  metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_document_output_elements_output
  ON document_output_elements(output_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_document_output_elements_engine_file
  ON document_output_elements(run_engine_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_document_output_elements_annotation
  ON document_output_elements(annotation_id);

CREATE TABLE IF NOT EXISTS document_output_spans (
  span_id TEXT PRIMARY KEY,
  output_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  run_engine_id TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  annotation_id TEXT,
  source_region_key TEXT,
  text_start UBIGINT NOT NULL,
  text_end UBIGINT NOT NULL,
  category TEXT NOT NULL DEFAULT 'text',
  metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_document_output_spans_output
  ON document_output_spans(output_id, text_start);
CREATE INDEX IF NOT EXISTS idx_document_output_spans_engine_file
  ON document_output_spans(run_engine_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_document_output_spans_annotation
  ON document_output_spans(annotation_id);
";
