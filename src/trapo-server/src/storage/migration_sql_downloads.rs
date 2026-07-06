pub(super) const DOWNLOAD_MANAGER_CONCURRENCY: &str = r"
ALTER TABLE download_events ADD COLUMN IF NOT EXISTS error_kind TEXT;
INSERT INTO settings(key, value, updated_at)
VALUES ('download_concurrency', '4'::JSON, current_timestamp)
ON CONFLICT(key) DO NOTHING;
";

pub(super) const INGEST_RUN_COMPLETION_MANIFESTS: &str = r"
CREATE TABLE IF NOT EXISTS ingest_run_completion_manifests (
  run_id TEXT PRIMARY KEY, completed_at TEXT NOT NULL, status TEXT NOT NULL,
  root_path TEXT NOT NULL, profile_id TEXT NOT NULL, engine_id TEXT NOT NULL,
  model_id TEXT NOT NULL, runtime_id TEXT NOT NULL,
  queued_files INTEGER NOT NULL, processed_pages INTEGER NOT NULL, total_pages INTEGER NOT NULL,
  file_count INTEGER NOT NULL, page_count INTEGER NOT NULL,
  summary_json TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_ingest_run_completion_completed_at
  ON ingest_run_completion_manifests(completed_at);
";
