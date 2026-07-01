pub struct Migration {
    pub id: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        id: 1,
        name: "initial_workbench_schema",
        sql: r#"
CREATE TABLE IF NOT EXISTS app_metadata (
  key TEXT PRIMARY KEY, value JSON NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY, value JSON NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS model_assets (
  model_id TEXT PRIMARY KEY, display_name TEXT NOT NULL, repo_id TEXT NOT NULL, filename TEXT NOT NULL,
  local_path TEXT, status TEXT NOT NULL, size_bytes UBIGINT, sha256 TEXT,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS files (
  file_hash TEXT PRIMARY KEY, display_name TEXT NOT NULL, extension TEXT NOT NULL,
  size_bytes UBIGINT NOT NULL, modified_at TIMESTAMP, page_count INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL, created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS file_locations (
  file_hash TEXT NOT NULL, root_path TEXT NOT NULL, absolute_path TEXT NOT NULL,
  relative_path TEXT NOT NULL, observed_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, absolute_path)
);
CREATE TABLE IF NOT EXISTS ingest_runs (
  run_id TEXT PRIMARY KEY, root_path TEXT NOT NULL, status TEXT NOT NULL, profile_id TEXT NOT NULL,
  engine_id TEXT NOT NULL, reprocess BOOLEAN NOT NULL DEFAULT false,
  started_at TIMESTAMP NOT NULL DEFAULT current_timestamp, finished_at TIMESTAMP, error TEXT
);
CREATE TABLE IF NOT EXISTS ingest_work_units (
  work_unit_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  status TEXT NOT NULL, attempts INTEGER NOT NULL DEFAULT 0, error TEXT,
  queued_at TIMESTAMP NOT NULL DEFAULT current_timestamp, started_at TIMESTAMP, finished_at TIMESTAMP,
  UNIQUE (run_id, file_hash, page_no)
);
CREATE TABLE IF NOT EXISTS document_pages (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, width_px INTEGER, height_px INTEGER,
  render_dpi INTEGER NOT NULL DEFAULT 200, status TEXT NOT NULL, error TEXT,
  PRIMARY KEY (file_hash, page_no)
);
CREATE TABLE IF NOT EXISTS document_preview_images (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, variant TEXT NOT NULL, path TEXT NOT NULL,
  width_px INTEGER NOT NULL, height_px INTEGER NOT NULL, created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, variant)
);
CREATE TABLE IF NOT EXISTS ocr_documents (
  file_hash TEXT PRIMARY KEY, engine_id TEXT NOT NULL, profile_id TEXT NOT NULL,
  runtime_metadata JSON, status TEXT NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS document_page_ocr (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, engine_id TEXT NOT NULL, profile_id TEXT NOT NULL,
  raw_text TEXT NOT NULL, cleaned_text TEXT NOT NULL, status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 1, error TEXT, elapsed_ms INTEGER, options JSON,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, engine_id, profile_id)
);
CREATE TABLE IF NOT EXISTS document_regions (
  region_id TEXT PRIMARY KEY, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL, profile_id TEXT NOT NULL, label TEXT NOT NULL,
  bbox_kind TEXT NOT NULL DEFAULT 'TOPLEFT_NORMALIZED_0_999',
  x1 DOUBLE NOT NULL, y1 DOUBLE NOT NULL, x2 DOUBLE NOT NULL, y2 DOUBLE NOT NULL,
  source_span_start UBIGINT, source_span_end UBIGINT
);
CREATE TABLE IF NOT EXISTS document_text_region_links (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, region_id TEXT NOT NULL,
  text_start UBIGINT NOT NULL, text_end UBIGINT NOT NULL,
  PRIMARY KEY (file_hash, page_no, region_id, text_start, text_end)
);
CREATE TABLE IF NOT EXISTS document_terms (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, term TEXT NOT NULL,
  text_start UBIGINT NOT NULL, text_end UBIGINT NOT NULL
);
CREATE TABLE IF NOT EXISTS annotation_visibility_overrides (
  file_hash TEXT NOT NULL, page_no INTEGER NOT NULL, region_id TEXT NOT NULL,
  hidden BOOLEAN NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, region_id)
);
CREATE TABLE IF NOT EXISTS annotation_style_settings (
  key TEXT PRIMARY KEY, value JSON NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS ingest_diagnostic_spans (
  span_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, parent_span_id TEXT, name TEXT NOT NULL,
  started_at TIMESTAMP NOT NULL, finished_at TIMESTAMP, attributes JSON
);
CREATE TABLE IF NOT EXISTS ingest_diagnostic_events (
  event_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, span_id TEXT, level TEXT NOT NULL,
  message TEXT NOT NULL, attributes JSON, created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS ingest_model_leases (
  lease_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, model_id TEXT NOT NULL,
  acquired_at TIMESTAMP NOT NULL DEFAULT current_timestamp, released_at TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_regions_file_page ON document_regions(file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_terms_term ON document_terms(term);
CREATE INDEX IF NOT EXISTS idx_work_units_run_status ON ingest_work_units(run_id, status);
"#,
    },
    Migration {
        id: 2,
        name: "ocr_dashboard_persistence_details",
        sql: r#"
ALTER TABLE files ADD COLUMN IF NOT EXISTS error TEXT;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS queued_files INTEGER DEFAULT 0;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS processed_pages INTEGER DEFAULT 0;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS total_pages INTEGER DEFAULT 0;
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS content_markdown TEXT;
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS content_html TEXT;
CREATE INDEX IF NOT EXISTS idx_files_status_updated ON files(status, updated_at);
CREATE INDEX IF NOT EXISTS idx_page_ocr_file_page ON document_page_ocr(file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_page_ocr_text ON document_page_ocr(file_hash, cleaned_text);
"#,
    },
    Migration {
        id: 3,
        name: "model_selection_persistence",
        sql: r#"
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS model_id TEXT DEFAULT 'unlimited-ocr-q4-k-m';
INSERT INTO settings(key, value, updated_at)
VALUES ('selected_model_id', '"unlimited-ocr-q4-k-m"'::JSON, current_timestamp)
ON CONFLICT(key) DO NOTHING;
"#,
    },
    Migration {
        id: 4,
        name: "runtime_selection_persistence",
        sql: r#"
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS runtime_id TEXT;
INSERT INTO settings(key, value, updated_at)
VALUES ('selected_runtime_id', '""'::JSON, current_timestamp)
ON CONFLICT(key) DO NOTHING;
INSERT INTO settings(key, value, updated_at)
VALUES ('selected_profile_id', '"experimental-exact-prefill-q4"'::JSON, current_timestamp)
ON CONFLICT(key) DO NOTHING;
"#,
    },
    Migration {
        id: 5,
        name: "workbench_ui_and_region_annotations",
        sql: r#"
CREATE TABLE IF NOT EXISTS document_region_annotations (
  region_id TEXT PRIMARY KEY, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  content_markdown TEXT NOT NULL, content_html TEXT, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
INSERT INTO document_region_annotations(region_id, file_hash, page_no, content_markdown, content_html, updated_at)
SELECT region_id, file_hash, page_no, coalesce(content_markdown, label), content_html, current_timestamp
FROM document_regions
WHERE content_markdown IS NOT NULL
ON CONFLICT(region_id) DO NOTHING;
CREATE INDEX IF NOT EXISTS idx_region_annotations_file_page ON document_region_annotations(file_hash, page_no);
INSERT INTO settings(key, value, updated_at)
VALUES (
  'workbench_ui',
  '{"theme":"dark","auto_follow_regions":true,"overlay_visible":true,"labels_visible":true,"panes_collapsed":{"explorer":false,"details":true,"diagnostics":true}}'::JSON,
  current_timestamp
)
ON CONFLICT(key) DO NOTHING;
"#,
    },
    Migration {
        id: 6,
        name: "ocr_page_metrics",
        sql: r#"
CREATE TABLE IF NOT EXISTS ocr_page_metrics (
  run_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL, profile_id TEXT NOT NULL, model_id TEXT NOT NULL,
  runtime_id TEXT, runtime_platform TEXT, accelerator TEXT, status TEXT NOT NULL, error TEXT,
  token_count UBIGINT NOT NULL DEFAULT 0, chunk_count UBIGINT NOT NULL DEFAULT 0,
  first_token_latency_ms UBIGINT NOT NULL DEFAULT 0, generation_duration_ms UBIGINT NOT NULL DEFAULT 0,
  elapsed_ms UBIGINT NOT NULL DEFAULT 0, min_tps DOUBLE NOT NULL DEFAULT 0,
  max_tps DOUBLE NOT NULL DEFAULT 0, avg_tps DOUBLE NOT NULL DEFAULT 0,
  started_at TEXT NOT NULL, first_token_at TEXT, completed_at TEXT,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (run_id, file_hash, page_no)
);
CREATE INDEX IF NOT EXISTS idx_ocr_page_metrics_run ON ocr_page_metrics(run_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_ocr_page_metrics_model_runtime ON ocr_page_metrics(model_id, runtime_id);
"#,
    },
];
