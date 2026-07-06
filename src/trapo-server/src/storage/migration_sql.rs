pub(super) const UUID_V7_IDENTITY_AND_ANNOTATIONS: &str = r"
CREATE TABLE IF NOT EXISTS persistence_id_migrations (
  id_kind TEXT NOT NULL, old_id TEXT NOT NULL, new_id TEXT NOT NULL, migrated_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (id_kind, old_id)
);

CREATE TABLE IF NOT EXISTS document_annotation_identities (
  annotation_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, file_hash TEXT NOT NULL, page_no INTEGER NOT NULL,
  engine_id TEXT NOT NULL, profile_id TEXT NOT NULL, source_region_key TEXT NOT NULL, discovery_index INTEGER NOT NULL,
  label TEXT NOT NULL, bbox_kind TEXT NOT NULL DEFAULT 'TOPLEFT_NORMALIZED_0_999',
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
