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
/api/documents/{file_hash}/text
/api/documents/{file_hash}/regions
/api/documents/{file_hash}/regions/{region_id}/snippet
/scalar
```

Text region spans are zero-width anchors. Region content is scoped from one
anchor to the next, and image-like regions can expose cropped local PNG snippets
through the snippet endpoint.

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
