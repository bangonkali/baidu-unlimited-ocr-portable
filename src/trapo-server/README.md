# trapo-server

`trapo-server` is the Rust/Axum backend for Trapo, a local-first RAG Ingest
Workbench for PDFs, images, and supported document formats. It serves the React
app from `src/trapo-client/dist`, exposes the JSON API, persists ingest state in
DuckDB, renders PDFs through PDFium, and streams live OCR text and bounding boxes
to the client.

Run locally:

```sh
cargo run -p trapo-server -- --port 8890 --no-browser
```

Useful endpoints:

```text
/api/status
/api/openapi.json
/api/ocr/events
/api/diagnostics/runs
/api/diagnostics/trace
/api/diagnostics/progress
/api/diagnostics/analytics
/api/diagnostics/models
/api/documents/{file_hash}/text
/api/documents/{file_hash}/regions
/api/documents/{file_hash}/regions/{region_id}/snippet
/scalar
```

Text region spans are zero-width anchors. Region content is scoped from one
anchor to the next, and image-like regions can expose cropped local PNG snippets
through the snippet endpoint.

Realtime `ocr.page.*` events are persisted before websocket broadcast. The
client can replay historical page events from `/api/ocr/events` and then merge
new websocket events, while completed pages are rebuilt from persisted page,
box, and span rows.

Page numbers are a numeric contract. Rust DTOs use `u32`, OpenAPI emits integer
schemas for `page_no`, `page_count`, and `current_page`, and DuckDB page columns
use `INTEGER`. UI labels may render `Page 10`, but clients should sort and
filter using the numeric fields.

Diagnostics are stored as work units, spans, events, and model leases in
DuckDB. The Workbench diagnostics route queries those rows as a waterfall tree,
run progress tree, analytics summary, and model lease history.

The server writes:

```text
data/trapo.duckdb
logs/trapo-server.log
```

OCR inference loads the native `uocr-ffi` runtime from
`thirdparty/uocr-runtime/<platform>/bin`. That ABI name is internal; the public
application and release artifacts are Trapo.

`PDFium-rs` is GPL-3.0 licensed. Trapo server distributions that include or link
PDFium/PDFium-rs must preserve the corresponding GPL obligations and notices.
