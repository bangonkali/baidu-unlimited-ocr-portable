# C++ Drogon Workbench Port

The target product path is a local-first C++/React workbench:

- `src/uocr-server`: C++20 service code, optional Drogon API executable,
  checked-in OpenAPI, DuckDB migration SQL, OCR parser, filesystem scanning,
  and the `uocr-ffi` engine adapter.
- `src/uocr-client`: React 19 SPA with VS Code-style workbench panes,
  TanStack Query/Router/Store/Table/Virtual/Pacer, CSS modules, Storybook,
  Biome, and Orval.
- `src/baidu_unlimited_ocr_portable`: Python Gradio/reference demo. It remains
  useful for behavior comparison and is not the launched product runtime.

## Backend Shape

The server is structured around small service boundaries:

- core: profile catalog, OCR marker parsing, stable region IDs, runaway-output
  detection, and overlay conversion.
- fs: trusted folder validation and recursive supported-file discovery.
- storage: ordered DuckDB migrations for files, runs, page OCR, regions,
  text-region links, diagnostics, settings, and model leases.
- ocr: abstract `OcrEngine` plus `UnlimitedOcrFfiEngine`, which loads
  `thirdparty/uocr-runtime/<platform>/bin/uocr-ffi.*`.
- render: `PageRenderer` with an embedded MuPDF implementation linked into
  `uocr-server.exe`. PDF pages are rendered to cached PNG files at 200 DPI and
  reused for preview and OCR.
- download: native libcurl/OpenSSL Hugging Face downloads with environment-only
  `HF_TOKEN` / `HUGGING_FACE_HUB_TOKEN` auth, resumable temp files, SHA256
  verification when metadata provides it, per-file progress, speed, ETA,
  cancellation, and typed realtime updates.
- realtime: a single in-process event hub plus Drogon websocket controller at
  `/api/events`. The websocket sends typed JSON envelopes for model progress,
  run status, document/page changes, parsed regions, cleaned text, status, and
  appended logs.
- app: optional Drogon route registration, websocket registration, and static
  OpenAPI serving.

Drogon is optional at configure time so parser/schema/scanner tests can run on
machines that have not installed Drogon yet. Build the core validation path:

```powershell
cmake -S . -B build/uocr-server -DUOCR_BUILD_SERVER=OFF -DUOCR_BUILD_TESTS=ON
cmake --build build/uocr-server --config Release --target uocr-core-tests
ctest --test-dir build/uocr-server -C Release --output-on-failure
```

When Drogon is available through CMake package discovery, enable the executable:

```powershell
cmake -S . -B build/uocr-server-drogon -DUOCR_BUILD_SERVER=ON
cmake --build build/uocr-server-drogon --config Release --target uocr-server
```

Release builds inject `UOCR_VERSION` from the tag. The executable reports that
metadata without starting the server:

```powershell
uocr-server.exe --version
```

The default runtime command binds `127.0.0.1:8765`, serves `/api/*` plus
`/api/openapi.json`, and falls back to `web/index.html` for the React SPA.

## API Contract

`src/uocr-server/openapi/uocr.openapi.json` is the API source of truth. It
covers the currently implemented workbench surface: health/status, trusted
folder selection, model download state, model download cancel/events, ingest
start/stop, run snapshots/events, document lists/details, regions, text spans,
preview images, recent logs, and settings. Removed or future-only surfaces are
intentionally omitted from the UI and OpenAPI contract until they work end to
end.

The React client runs Orval against that file:

```powershell
cd src\uocr-client
bun run generate-api
```

## Realtime Contract

The UI opens one websocket to `/api/events` after startup. Frontend commands and
mutations continue to use the OpenAPI HTTP routes; the websocket is receive-only
for backend state changes.

Each websocket frame is a compact JSON envelope:

```json
{
  "version": 1,
  "sequence": 12,
  "type": "document.regions.changed",
  "occurred_at": "2026-06-30T00:00:00Z",
  "payload": {}
}
```

Current event types are `connection.ready`, `status.changed`, `model.changed`,
`run.changed`, `document.changed`, `document.page.changed`,
`document.regions.changed`, `document.text.changed`, and `log.appended`.

These events drive immediate UI updates for the Models panel, ingest toolbar,
explorer tree, preview overlays, text pane, diagnostics logs, and status bar.
The client still keeps ordinary HTTP queries for initial loads, refresh, and
fallback refetching.

## Traceability Model

Unlimited-OCR markers are parsed from raw model output:

```text
<|ref|>Label<|/ref|><|det|>[[x1,y1,x2,y2]]<|/det|>
```

Boxes are stored as `TOPLEFT_NORMALIZED_0_999`. API overlays expose percentages
for rendering. Region IDs are stable FNV-1a hashes over file hash, page, engine,
profile, source marker span, label, and normalized bounding box.

Cleaned text removes OCR markers while preserving text spans. The
`document_text_region_links` table maps cleaned text offsets back to
`document_regions`, allowing text-span selection to focus the overlay and
overlay selection to focus the corresponding text.
