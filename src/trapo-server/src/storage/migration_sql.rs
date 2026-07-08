pub(super) const UUID_V7_IDENTITY_AND_ANNOTATIONS: &str = r"
CREATE TABLE IF NOT EXISTS persistence_id_migrations (
  id_kind TEXT NOT NULL, old_id TEXT NOT NULL, new_id TEXT NOT NULL, migrated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (id_kind, old_id)
);

CREATE TABLE IF NOT EXISTS document_annotation_identities (
  annotation_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL, profile_id TEXT NOT NULL, source_region_key TEXT NOT NULL, discovery_index INTEGER NOT NULL,
  label TEXT NOT NULL, bbox_kind TEXT NOT NULL DEFAULT 'axis_aligned',
  x1 DOUBLE NOT NULL, y1 DOUBLE NOT NULL, x2 DOUBLE NOT NULL, y2 DOUBLE NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp, updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_annotation_identity_source ON document_annotation_identities(run_id, file_hash, page_no, source_region_key);
CREATE INDEX IF NOT EXISTS idx_annotation_identity_file_page ON document_annotation_identities(file_hash, page_no);

ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS annotation_id TEXT;
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS source_region_key TEXT;
UPDATE document_regions SET annotation_id = region_id WHERE annotation_id IS NULL OR annotation_id = '';
UPDATE document_regions SET source_region_key = region_id WHERE source_region_key IS NULL OR source_region_key = '';

ALTER TABLE document_text_region_links ADD COLUMN IF NOT EXISTS annotation_id TEXT;
UPDATE document_text_region_links SET annotation_id = region_id WHERE annotation_id IS NULL OR annotation_id = '';

ALTER TABLE download_events ADD COLUMN IF NOT EXISTS download_key TEXT DEFAULT '';
UPDATE download_events SET download_key = download_id WHERE download_key IS NULL OR download_key = '';
ALTER TABLE download_events ADD COLUMN IF NOT EXISTS error_kind TEXT;

ALTER TABLE ocr_stream_events ADD COLUMN IF NOT EXISTS event_id TEXT;
UPDATE ocr_stream_events SET event_id = CAST(sequence AS VARCHAR)
WHERE event_id IS NULL OR event_id = '';
CREATE UNIQUE INDEX IF NOT EXISTS idx_ocr_stream_events_event_id ON ocr_stream_events(event_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_ingest_work_units_run_work_key ON ingest_work_units(run_id, work_key);
CREATE UNIQUE INDEX IF NOT EXISTS idx_ingest_model_leases_run_execution_key ON ingest_model_leases(run_id, execution_key);
";

pub(super) const RUN_SCOPED_OCR_OUTPUTS: &str = r"
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS run_id TEXT;
ALTER TABLE document_text_region_links ADD COLUMN IF NOT EXISTS run_id TEXT;

UPDATE document_regions
SET run_id = (
  SELECT a.run_id
  FROM document_annotation_identities a
  WHERE a.annotation_id = document_regions.annotation_id
  LIMIT 1
)
WHERE (run_id IS NULL OR run_id = '')
  AND annotation_id IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM document_annotation_identities a
    WHERE a.annotation_id = document_regions.annotation_id
  );

UPDATE document_text_region_links
SET run_id = (
  SELECT r.run_id
  FROM document_regions r
  WHERE r.annotation_id = document_text_region_links.annotation_id
     OR r.region_id = document_text_region_links.region_id
  LIMIT 1
)
WHERE (run_id IS NULL OR run_id = '')
  AND EXISTS (
    SELECT 1
    FROM document_regions r
    WHERE r.annotation_id = document_text_region_links.annotation_id
       OR r.region_id = document_text_region_links.region_id
  );

CREATE TABLE IF NOT EXISTS document_run_page_ocr (
  run_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL, profile_id TEXT NOT NULL,
  raw_text TEXT NOT NULL, cleaned_text TEXT NOT NULL, status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 1, error TEXT, elapsed_ms INTEGER, options JSON,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (run_id, file_hash, page_no, engine_id, profile_id)
);

INSERT INTO document_run_page_ocr(
  run_id, file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text,
  status, attempts, error, elapsed_ms, options, created_at, updated_at
)
WITH scoped AS (
  SELECT
    coalesce(
      (
        SELECT m.run_id
        FROM ocr_page_metrics m
        WHERE m.file_hash = p.file_hash
          AND m.page_no = p.page_no
          AND m.engine_id = p.engine_id
          AND m.profile_id = p.profile_id
        ORDER BY m.updated_at DESC
        LIMIT 1
      ),
      (
        SELECT r.run_id
        FROM document_regions r
        WHERE r.file_hash = p.file_hash
          AND r.page_no = p.page_no
          AND r.engine_id = p.engine_id
          AND r.profile_id = p.profile_id
          AND r.run_id IS NOT NULL
          AND r.run_id <> ''
        LIMIT 1
      ),
      ''
    ) AS scoped_run_id,
    p.file_hash, p.page_no, p.engine_id, p.profile_id, p.raw_text, p.cleaned_text,
    p.status, p.attempts, p.error, p.elapsed_ms, p.options, p.created_at
  FROM document_page_ocr p
)
SELECT
  scoped_run_id, file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text,
  status, attempts, error, elapsed_ms, options, created_at, current_timestamp
FROM scoped
WHERE scoped_run_id <> ''
ON CONFLICT DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_document_run_page_ocr_run_file_page
  ON document_run_page_ocr(run_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_regions_run_file_page
  ON document_regions(run_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_text_region_links_run_file_page
  ON document_text_region_links(run_id, file_hash, page_no);
";

pub(super) const RAG_PIPELINE: &str = r"
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS category TEXT;
UPDATE document_regions SET category = label WHERE category IS NULL OR category = '';

ALTER TABLE document_annotation_identities ADD COLUMN IF NOT EXISTS category TEXT;
UPDATE document_annotation_identities SET category = label WHERE category IS NULL OR category = '';

CREATE TABLE IF NOT EXISTS pipeline_tasks (
  task_id TEXT PRIMARY KEY,
  task_kind TEXT NOT NULL,
  origin_run_id TEXT,
  status TEXT NOT NULL,
  params_json TEXT NOT NULL DEFAULT '{}',
  result_json TEXT NOT NULL DEFAULT '{}',
  queued_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  runner_id TEXT,
  error TEXT
);
CREATE INDEX IF NOT EXISTS idx_pipeline_tasks_status ON pipeline_tasks(status, queued_at);
CREATE INDEX IF NOT EXISTS idx_pipeline_tasks_origin ON pipeline_tasks(origin_run_id, task_kind, queued_at);

CREATE TABLE IF NOT EXISTS rag_embedding_models (
  model_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  provider TEXT NOT NULL,
  repo_id TEXT NOT NULL,
  filename TEXT NOT NULL,
  revision TEXT NOT NULL,
  routing_origin TEXT NOT NULL,
  model_family TEXT NOT NULL,
  dimension INTEGER NOT NULL,
  context_tokens INTEGER NOT NULL,
  pooling TEXT NOT NULL,
  normalize BOOLEAN NOT NULL,
  query_prefix TEXT NOT NULL DEFAULT '',
  document_prefix TEXT NOT NULL DEFAULT '',
  llama_params_json TEXT NOT NULL DEFAULT '{}',
  recommended_vram_gb DOUBLE NOT NULL DEFAULT 4,
  active BOOLEAN NOT NULL DEFAULT true,
  created_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR),
  updated_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR)
);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_models_origin ON rag_embedding_models(routing_origin, active);

CREATE TABLE IF NOT EXISTS rag_text_segments (
  segment_id TEXT PRIMARY KEY,
  source_run_id TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  page_no INTEGER NOT NULL,
  segment_index INTEGER NOT NULL,
  annotation_id TEXT,
  category TEXT NOT NULL,
  text TEXT NOT NULL,
  token_estimate INTEGER NOT NULL,
  text_start UBIGINT NOT NULL DEFAULT 0,
  text_end UBIGINT NOT NULL DEFAULT 0,
  source_kind TEXT NOT NULL DEFAULT 'page',
  created_at TEXT NOT NULL DEFAULT CAST(now() AS VARCHAR),
  UNIQUE(source_run_id, file_hash, page_no, segment_index)
);
CREATE INDEX IF NOT EXISTS idx_rag_text_segments_run ON rag_text_segments(source_run_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_text_segments_annotation ON rag_text_segments(annotation_id);

CREATE TABLE IF NOT EXISTS rag_text_index_runs (
  text_index_run_id TEXT PRIMARY KEY,
  task_id TEXT,
  source_run_id TEXT NOT NULL,
  status TEXT NOT NULL,
  segments_indexed INTEGER NOT NULL DEFAULT 0,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  error TEXT
);
CREATE INDEX IF NOT EXISTS idx_rag_text_index_runs_source ON rag_text_index_runs(source_run_id, started_at);

CREATE TABLE IF NOT EXISTS rag_fts_index_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  text_index_run_id TEXT NOT NULL,
  source_run_id TEXT NOT NULL,
  index_name TEXT NOT NULL,
  status TEXT NOT NULL,
  segments_indexed INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  error TEXT
);
CREATE INDEX IF NOT EXISTS idx_rag_fts_snapshots_source ON rag_fts_index_snapshots(source_run_id, created_at);

CREATE TABLE IF NOT EXISTS rag_embedding_runs (
  embedding_run_id TEXT PRIMARY KEY,
  task_id TEXT,
  source_run_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  requested_dimension INTEGER NOT NULL,
  actual_dimension INTEGER NOT NULL,
  status TEXT NOT NULL,
  segments_total INTEGER NOT NULL DEFAULT 0,
  segments_embedded INTEGER NOT NULL DEFAULT 0,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  error TEXT,
  params_json TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_runs_source ON rag_embedding_runs(source_run_id, model_id, started_at);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_runs_model_status ON rag_embedding_runs(model_id, status);

CREATE TABLE IF NOT EXISTS rag_embedding_vectors_128 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[128] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_256 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[256] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_512 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[512] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_768 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[768] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_1024 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[1024] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_2560 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[2560] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);
CREATE TABLE IF NOT EXISTS rag_embedding_vectors_4096 (
  embedding_run_id TEXT NOT NULL, source_run_id TEXT NOT NULL, segment_id TEXT NOT NULL,
  model_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  embedding FLOAT[4096] NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}', created_at TEXT NOT NULL,
  PRIMARY KEY (embedding_run_id, segment_id)
);

CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_128_lookup ON rag_embedding_vectors_128(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_256_lookup ON rag_embedding_vectors_256(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_512_lookup ON rag_embedding_vectors_512(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_768_lookup ON rag_embedding_vectors_768(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_1024_lookup ON rag_embedding_vectors_1024(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_2560_lookup ON rag_embedding_vectors_2560(source_run_id, model_id, file_hash, page_no);
CREATE INDEX IF NOT EXISTS idx_rag_embedding_vectors_4096_lookup ON rag_embedding_vectors_4096(source_run_id, model_id, file_hash, page_no);
";

pub(super) const OCR_GEOMETRY_MODEL: &str = r"
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS geometry_json TEXT DEFAULT '{}';
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS coordinate_space TEXT DEFAULT 'page_percent';
ALTER TABLE document_regions ADD COLUMN IF NOT EXISTS rotation_degrees DOUBLE;
UPDATE document_regions
SET bbox_kind = 'axis_aligned'
WHERE bbox_kind IS NULL OR bbox_kind = '' OR bbox_kind = 'TOPLEFT_NORMALIZED_0_999';
UPDATE document_regions
SET coordinate_space = 'page_percent'
WHERE coordinate_space IS NULL OR coordinate_space = '';
UPDATE document_regions
SET geometry_json = '{}'
WHERE geometry_json IS NULL OR geometry_json = '';

ALTER TABLE document_annotation_identities ADD COLUMN IF NOT EXISTS geometry_json TEXT DEFAULT '{}';
ALTER TABLE document_annotation_identities ADD COLUMN IF NOT EXISTS coordinate_space TEXT DEFAULT 'page_percent';
ALTER TABLE document_annotation_identities ADD COLUMN IF NOT EXISTS rotation_degrees DOUBLE;
UPDATE document_annotation_identities
SET bbox_kind = 'axis_aligned'
WHERE bbox_kind IS NULL OR bbox_kind = '' OR bbox_kind = 'TOPLEFT_NORMALIZED_0_999';
UPDATE document_annotation_identities
SET coordinate_space = 'page_percent'
WHERE coordinate_space IS NULL OR coordinate_space = '';
UPDATE document_annotation_identities
SET geometry_json = '{}'
WHERE geometry_json IS NULL OR geometry_json = '';
";
