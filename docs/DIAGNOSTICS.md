# Diagnostics And Replay

Trapo records ingest diagnostics in DuckDB so the Workbench can inspect a run as
a waterfall, a progress tree, aggregate analytics, model leases, and log rows.
The goal is to make OCR runs explainable enough for later Graph RAG ingestion,
where page text, regions, spans, timing, and failures must be traceable.

## Stored Data

`trapo-server` persists diagnostics in structured tables:

- `ingest_runs` stores each OCR run and its status.
- `ingest_run_documents` stores the ordered files discovered for each run.
- `ingest_work_units` stores run, file, page, and phase progress.
- `ingest_diagnostic_spans` stores waterfall spans with timing, status, engine,
  pipeline step, parent span, and JSON attributes.
- `ingest_diagnostic_events` stores logs and structured events tied to a run,
  file, page, or span.
- `ingest_model_leases` stores the model/runtime/profile selected for a run.
- `ocr_stream_events` stores each replayable page stream event before broadcast.

These rows are written through repository methods rather than direct SQL in
route handlers. Query endpoints shape the data for the React tree grid and keep
the DuckDB schema independent from UI rendering details.

Page fields are stored as numeric data. DuckDB page columns, including
`files.page_count`, page-scoped `page_no` columns, diagnostics page filters, and
OCR replay event scopes, use `INTEGER` storage. API contracts keep the same
numeric shape so diagnostics can filter and sort by page number without string
coercion.

## Waterfall View

The diagnostics route loads runs from `/api/diagnostics/runs` and detailed trace
data from `/api/diagnostics/trace`. The Workbench renders span rows through the
shared `TreeGrid` component. Each row shows the span name, page number when
available, status icon, duration badge, and child spans.

Progress data comes from `/api/diagnostics/progress` and is grouped by run and
work unit. Analytics come from `/api/diagnostics/analytics` and summarize slow
spans, status breakdowns, pipeline-step breakdowns, and errors. Model lease
history comes from `/api/diagnostics/models`.

## OCR Replay

Live OCR emits typed events such as:

```text
ocr.page.stream.started
ocr.page.text.patch
ocr.page.region.upsert
ocr.page.span.upsert
ocr.page.stream.completed
```

The realtime hub stores replayable `ocr.page.*` events before publishing them
to the websocket. `/api/ocr/events` returns historical events filtered by run,
file hash, page number, sequence, and limit.

The React workbench hydrates the selected active page by projecting stored events
with the same reducer primitives used for websocket events. Projection is
idempotent, so repeated replay reads do not duplicate appended text. After
hydration, new websocket events continue from the current state. If the client
detects a websocket sequence gap, it reads missed `ocr.page.*` rows from
DuckDB and invalidates table-backed run, document, status, and log queries.
Completed pages are loaded from the persisted document/page/region tables so
refresh after completion produces the same preview and bounding boxes without
relying on websocket history alone.

## Failure Analysis

Document rendering, page OCR, fallback OCR, cancellation, and page failures all
record spans or diagnostic events. The diagnostics page should be the first
place to verify whether a slow run spent time rendering PDFs, waiting for the
runtime, processing a specific page, or failing a model/runtime step.
