# Trapo DuckDB Query Inventory and Test Coverage

Runtime storage query call sites: `62`
Migration SQL bundles: `11`

This file is the required manifest for DuckDB-backed reads and writes in the
Trapo server. Any new storage query, API route backed by storage, or migration
must update this file and must add a DuckDB-backed positive, negative, and
boundary test before merge or release.

Coverage terms:
- Positive: expected row is written, updated, or returned.
- Negative: missing key, ignored event, duplicate id, no-op update, or empty
  result is exercised.
- Boundary: empty input, `limit=0`, duplicate conflict, UUID v7 migration
  idempotency, nullable bytes, or integer saturation is exercised.

## Runtime Queries

| ID | Direction | Source methods | Tables and query summary | Positive coverage | Negative coverage | Boundary coverage |
| --- | --- | --- | --- | --- | --- | --- |
| DB-OPEN-001 | read/write | `open`, `migrate_generated_ids_to_uuid_v7` | Opens DuckDB, creates `schema_migrations`, selects applied migration ids, executes migration SQL, inserts migration records, runs UUID v7 migration. | `migrates_and_persists_settings`; `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | Re-opening an already migrated DB skips applied migrations. | Nested database path creation; migration idempotency. |
| DB-SET-001 | read/write | `setting_value`, `put_setting` | Reads and upserts `settings` JSON by key. | `migrates_and_persists_settings`; `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Missing setting returns `None`. | JSON value overwrite. |
| DB-SHUTDOWN-001 | write | `checkpoint` | Runs a DuckDB checkpoint during clean server shutdown after realtime and annotation queues drain. | `shutdown_route_requires_confirmation_and_blocks_new_work` | Shutdown without active work still completes. | Empty queue drain plus checkpoint is idempotent. |
| DB-RUN-001 | write | `upsert_run` | Upserts `ingest_runs`, including terminal `finished_at`. | `reloads_run_document_membership`; `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Missing stop route is covered at API level by not-found behavior. | Status conflict update from queued to completed. |
| DB-RUN-DOC-001 | write | `replace_run_documents` | Deletes and re-inserts `ingest_run_documents` with ordinals. | `reloads_run_document_membership`; `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Missing run delete is a no-op. | Empty file list is accepted. |
| DB-DOC-001 | write/read | `upsert_document`, `search_document_hashes` | Upserts `files` and `file_locations`; searches file name, relative path, and OCR text. | `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Search miss returns empty. | Search `limit=0` returns empty. |
| DB-PAGE-001 | write/read | `upsert_page`, `load_snapshot` | Upserts `document_pages` and source `document_preview_images`; snapshot loads runs, run documents, files, pages, boxes, and spans. | `reloads_page_regions_and_spans`; `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Empty snapshot paths return empty vectors in fresh DB tests. | `preview_path=None` does not erase existing preview row. |
| DB-OCR-001 | write/read | `replace_page_ocr`, `load_snapshot`, `load_document_regions_for_run`, `load_document_text_for_run` | Upserts `ocr_documents`, compatibility `document_page_ocr`, and run-scoped `document_run_page_ocr`; deletes stale links/regions only for the current run; inserts run-scoped `document_regions`, `document_region_annotations`, and `document_text_region_links`; reads text and overlays by `run_id` plus `file_hash`. | `reloads_page_regions_and_spans`; `core_document_page_and_metric_queries_cover_empty_updates_and_limits`; `run_scoped_document_reads_do_not_mix_ocr_outputs` | Replacing with no boxes removes stale boxes and spans for that run; missing run returns empty text. | Empty OCR boxes/spans are valid; same file/page can keep separate OCR outputs for multiple runs. |
| DB-ANNOT-001 | read/write | `persist_discovered_annotations` | Resolves existing or preassigned annotation identity; upserts `document_annotation_identities`, discovered `document_regions`, and text links through batched transactions. | `persists_discovered_annotation_identity_before_text_completion`; `persists_preassigned_annotation_identity_batches` | Empty batch is a no-op; re-discovering the same source key does not allocate a second id. | Same source key updates label/content while preserving UUID v7; multi-region batch persists distinct ids. |
| DB-METRIC-001 | read/write | `upsert_page_metrics`, `list_page_metrics` | Upserts and lists `ocr_page_metrics` by run or globally. | `core_document_page_and_metric_queries_cover_empty_updates_and_limits` | Missing run returns empty. | `limit=0` returns empty; `u64::MAX` saturates to `i64::MAX`. |
| DB-REPLAY-001 | read/write | `persist_realtime_event`, `persist_realtime_events`, `list_ocr_stream_events` | Inserts OCR page events into `ocr_stream_events`; lists by run, file, page, sequence, and limit. | `persists_and_lists_ocr_stream_events`; `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | Non-`ocr.page.*` event is ignored; missing run returns empty. | `limit=0` clamps to one; duplicate sequence is ignored. |
| DB-DOWNLOAD-001 | read/write | `insert_download_event`, `download_event_count` | Inserts `download_events`; test helper counts events by download id and type. | `persists_download_lifecycle_events`; `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | Duplicate `event_id` is ignored; missing download count is zero. | `total_bytes=NULL` is accepted. |
| DB-DIAG-WORK-001 | read/write | `upsert_work_unit`, `start_work_unit`, `finish_work_unit`, `diagnostic_work_units` | Upserts and transitions `ingest_work_units`; lists work units with file/location joins. | `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | Missing start/finish is a no-op; missing run filter returns empty. | `limit=0` clamps to one. |
| DB-DIAG-SPAN-001 | read/write | `insert_diagnostic_span`, `diagnostic_trace` | Upserts `ingest_diagnostic_spans`; filters by run, file, page, status, text query, and limit. | `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | No-match query returns empty. | `limit=0` clamps to one. |
| DB-DIAG-EVENT-001 | read/write | `insert_diagnostic_event`, `diagnostic_trace` | Inserts `ingest_diagnostic_events`; filters by run, file, page, query, and limit. | `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | No-match query returns empty. | `limit=0` clamps to one. |
| DB-DIAG-LEASE-001 | read/write | `insert_model_lease`, `diagnostic_model_leases` | Upserts `ingest_model_leases` by run and execution key; lists by run. | `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | Missing run filter returns empty. | Duplicate execution key updates the existing lease; `limit=0` clamps to one. |
| DB-DIAG-RUN-001 | read | `diagnostic_runs` | Aggregates `ingest_runs` with diagnostic spans for duration, span counts, errors, files, and pages. | `realtime_download_and_diagnostic_queries_cover_filters_duplicates_and_limits` | Empty DB returns empty through API smoke coverage. | `limit=0` clamps to one. |

## Migration Queries

| ID | Direction | Source methods | Tables and query summary | Positive coverage | Negative coverage | Boundary coverage |
| --- | --- | --- | --- | --- | --- | --- |
| DB-MIG-001 | read/write | `migrate_generated_ids_to_uuid_v7` | Reads distinct legacy run ids and updates `ingest_runs`, run documents, work units, diagnostic spans/events, model leases, metrics, and replay events. | `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | UUID v7 ids are skipped on second run. | Migration is idempotent. |
| DB-MIG-002 | read/write | `migrate_generated_ids_to_uuid_v7` | Reads legacy region ids and updates regions, annotation ids, text links, region annotations, and visibility overrides. | `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | UUID v7 ids are skipped on second run. | Markdown references are replaced. |
| DB-MIG-003 | read/write | `migrate_generated_ids_to_uuid_v7` | Migrates diagnostic span/event ids, work unit ids, model lease ids, download ids, and realtime event ids through `persistence_id_migrations`. | `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | Existing mapping is reused. | Duplicate migration run does not add rows. |
| DB-MIG-004 | read/write | `migrate_generated_ids_to_uuid_v7` | Replaces legacy ids inside `ocr_stream_events.payload_json`, `document_regions.content_markdown`, and `document_region_annotations.content_markdown`. | `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | Rows without the old id are untouched. | Payload and markdown replacement survives a second migration run. |
| DB-MIG-005 | read/write | `migrate_generated_ids_to_uuid_v7` | Backfills `document_annotation_identities` from existing `document_regions` and finds run ids through `ocr_page_metrics`. | `uuid_v7_migration_queries_cover_legacy_rows_payloads_and_backfills` | Existing annotation identity is not duplicated. | Backfilled id is UUID v7 and matches the migrated region id. |
| DB-MIG-006 | read/write | `migrate` | Adds run scope to `document_regions` and `document_text_region_links`, creates `document_run_page_ocr`, backfills run-scoped OCR rows from metrics or annotation identities, and indexes run/file/page reads. | `run_scoped_document_reads_do_not_mix_ocr_outputs` | Missing run reads return empty. | Existing file-level OCR rows remain available as compatibility cache while new rows are run-scoped. |

## API Routes Covered By DB Tests

Routes are listed here so route additions cannot bypass the query manifest:
`/api/health`, `/api/status`, `/api/openapi.json`, `/api/system/folder-dialog`,
`/api/system/shutdown`, `/api/ingest/start`, `/api/ingest/runs`, `/api/ingest/metrics/recent`,
`/api/ingest/runs/{run_id}`, `/api/ingest/runs/{run_id}/metrics`,
`/api/ingest/runs/{run_id}/stop`, `/api/ingest/runs/{run_id}/events`,
`/api/ocr/events`, `/api/diagnostics/runs`, `/api/diagnostics/trace`,
`/api/diagnostics/progress`, `/api/diagnostics/analytics`,
`/api/diagnostics/models`, `/api/documents`, `/api/search`,
`/api/documents/{file_hash}`, `/api/documents/{file_hash}/regions`,
`/api/documents/{file_hash}/regions/{region_id}/snippet`,
`/api/documents/{file_hash}/text`,
`/api/documents/{file_hash}/preview-images`,
`/api/documents/{file_hash}/preview-images/{variant}/{page_no}`,
`/api/settings`, `/api/models`, `/api/models/{model_id}/download`,
`/api/models/{model_id}/select`, `/api/models/{model_id}/cancel`,
`/api/models/{model_id}/events`, `/api/logs/recent`, `/api/events`.

API route smoke and contract tests live in `src/trapo-server/tests/api.rs`.
Database manifest guard tests live in `src/trapo-server/tests/db_manifest.rs`.
