#include "uocr/storage/migrations.hpp"

namespace uocr {

const std::vector<Migration>& duckdb_migrations() {
  static const std::vector<Migration> migrations = {
      {1, "initial_workbench_schema", R"SQL(
CREATE TABLE IF NOT EXISTS app_metadata (
  key TEXT PRIMARY KEY,
  value JSON NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value JSON NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS model_assets (
  model_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  repo_id TEXT NOT NULL,
  filename TEXT NOT NULL,
  local_path TEXT,
  status TEXT NOT NULL,
  size_bytes UBIGINT,
  sha256 TEXT,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS files (
  file_hash TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  extension TEXT NOT NULL,
  size_bytes UBIGINT NOT NULL,
  modified_at TIMESTAMP,
  page_count INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS file_locations (
  file_hash TEXT NOT NULL,
  root_path TEXT NOT NULL,
  absolute_path TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  observed_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, absolute_path)
);
CREATE TABLE IF NOT EXISTS ingest_runs (
  run_id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  status TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  engine_id TEXT NOT NULL,
  reprocess BOOLEAN NOT NULL DEFAULT false,
  started_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  finished_at TIMESTAMP,
  error TEXT
);
CREATE TABLE IF NOT EXISTS ingest_work_units (
  work_unit_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  queued_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  started_at TIMESTAMP,
  finished_at TIMESTAMP,
  UNIQUE (run_id, file_hash, page_no)
);
CREATE TABLE IF NOT EXISTS document_pages (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  width_px INTEGER,
  height_px INTEGER,
  render_dpi INTEGER NOT NULL DEFAULT 200,
  status TEXT NOT NULL,
  error TEXT,
  PRIMARY KEY (file_hash, page_no)
);
CREATE TABLE IF NOT EXISTS document_preview_images (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  variant TEXT NOT NULL,
  path TEXT NOT NULL,
  width_px INTEGER NOT NULL,
  height_px INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, variant)
);
CREATE TABLE IF NOT EXISTS ocr_documents (
  file_hash TEXT PRIMARY KEY,
  engine_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  runtime_metadata JSON,
  status TEXT NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS document_page_ocr (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  raw_text TEXT NOT NULL,
  cleaned_text TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 1,
  error TEXT,
  elapsed_ms INTEGER,
  options JSON,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, engine_id, profile_id)
);
CREATE TABLE IF NOT EXISTS document_regions (
  region_id TEXT PRIMARY KEY,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  label TEXT NOT NULL,
  bbox_kind TEXT NOT NULL DEFAULT 'TOPLEFT_NORMALIZED_0_999',
  x1 DOUBLE NOT NULL,
  y1 DOUBLE NOT NULL,
  x2 DOUBLE NOT NULL,
  y2 DOUBLE NOT NULL,
  source_span_start UBIGINT,
  source_span_end UBIGINT
);
CREATE TABLE IF NOT EXISTS document_text_region_links (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  region_id TEXT NOT NULL,
  text_start UBIGINT NOT NULL,
  text_end UBIGINT NOT NULL,
  PRIMARY KEY (file_hash, page_no, region_id, text_start, text_end)
);
CREATE TABLE IF NOT EXISTS document_terms (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  term TEXT NOT NULL,
  text_start UBIGINT NOT NULL,
  text_end UBIGINT NOT NULL
);
CREATE TABLE IF NOT EXISTS annotation_visibility_overrides (
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  region_id TEXT NOT NULL,
  hidden BOOLEAN NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (file_hash, page_no, region_id)
);
CREATE TABLE IF NOT EXISTS annotation_style_settings (
  key TEXT PRIMARY KEY,
  value JSON NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS ingest_diagnostic_spans (
  span_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  parent_span_id TEXT,
  name TEXT NOT NULL,
  started_at TIMESTAMP NOT NULL,
  finished_at TIMESTAMP,
  attributes JSON
);
CREATE TABLE IF NOT EXISTS ingest_diagnostic_events (
  event_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  span_id TEXT,
  level TEXT NOT NULL,
  message TEXT NOT NULL,
  attributes JSON,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE TABLE IF NOT EXISTS ingest_model_leases (
  lease_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  acquired_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  released_at TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_regions_file_page ON document_regions(file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_terms_term ON document_terms(term);
CREATE INDEX IF NOT EXISTS idx_work_units_run_status ON ingest_work_units(run_id, status);
)SQL"},
      {2, "ocr_dashboard_persistence_details", R"SQL(
ALTER TABLE files ADD COLUMN IF NOT EXISTS error TEXT;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS queued_files INTEGER DEFAULT 0;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS processed_pages INTEGER DEFAULT 0;
ALTER TABLE ingest_runs ADD COLUMN IF NOT EXISTS total_pages INTEGER DEFAULT 0;
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS content_markdown TEXT;
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS content_html TEXT;
CREATE INDEX IF NOT EXISTS idx_files_status_updated ON files(status, updated_at);
CREATE INDEX IF NOT EXISTS idx_page_ocr_file_page ON document_page_ocr(file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_page_ocr_text ON document_page_ocr(file_hash, cleaned_text);
)SQL"},
  };
  return migrations;
}

}  // namespace uocr
