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

- core: profile catalog, model catalog, OCR marker parsing, stable region IDs,
  runaway-output detection, and overlay conversion.
- fs: trusted folder validation and recursive supported-file discovery.
- storage: ordered DuckDB migrations for files, runs, page OCR, regions,
  text-region links, diagnostics, settings, and model leases.
- ocr: abstract `OcrEngine` plus `UnlimitedOcrFfiEngine`, which loads
  `thirdparty/uocr-runtime/<platform>/bin/uocr-ffi.*`.
- render: `PageRenderer` with an embedded MuPDF implementation linked into
  `uocr-server.exe`. PDF pages are rendered to cached PNG files at 200 DPI and
  reused for preview and OCR.
- download: native libcurl Hugging Face downloads for every compatible
  `sahilchachra/Unlimited-OCR-GGUF` catalog entry, with environment-only
  `HF_TOKEN` / `HUGGING_FACE_HUB_TOKEN` auth, resumable temp files, SHA256
  verification through the same vcpkg `OpenSSL::Crypto` dependency used by the
  server stack, per-file progress, speed, ETA, cancellation, and typed realtime
  updates.
- realtime: a single in-process event hub plus Drogon websocket controller at
  `/api/events`. The websocket sends typed JSON envelopes for model progress,
  run status, document/page changes, parsed regions, cleaned text, status, and
  appended logs.
- app: Drogon route registration, websocket registration, and static OpenAPI
  serving for the portable product executable.

Package dependencies are resolved through vcpkg by default. The local core
validation path can skip the server target, but it still uses the vcpkg
toolchain so curl/OpenSSL resolve the same way as the release package:

```powershell
cmake -S . -B build/uocr-server `
  -DCMAKE_TOOLCHAIN_FILE="$env:VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake" `
  -DUOCR_BUILD_SERVER=OFF `
  -DUOCR_BUILD_TESTS=ON
cmake --build build/uocr-server --config Release --target uocr-core-tests
ctest --test-dir build/uocr-server -C Release --output-on-failure
```

For the Windows portable product build, package dependencies come from the
vcpkg manifest. The current pinned baseline resolves Drogon `1.9.13` exactly
and OpenSSL `3.6.3` exactly; Trantor/Drogon keep OpenSSL TLS enabled, and SHA
verification links `OpenSSL::Crypto` from the same vcpkg install. CMake fails
early if the vcpkg toolchain is missing unless `UOCR_REQUIRE_VCPKG=OFF` is set
for one-off diagnostics.

Build the executable with the repository preset:

```powershell
cmake --preset windows-workbench
cmake --build --preset windows-workbench-release
```

Release builds inject `UOCR_VERSION` from the tag. The executable reports that
metadata without starting the server:

```powershell
uocr-server.exe --version
```

The default runtime command binds `127.0.0.1:8765`, serves `/api/*` plus
`/api/openapi.json`, and falls back to `web/index.html` for the React SPA.

## DuckDB Persistence

The workbench creates and migrates `data/uocr.duckdb` at server startup. The
Windows release uses the bundled DuckDB C API runtime (`duckdb.dll`) staged
beside `uocr-server.exe`; DuckDB is not built from source in the release
workflow.

The persisted OCR dashboard contract is:

- `files` and `file_locations`: file identity, display path, status, page count,
  size, latest observed root, and error text.
- `ingest_runs` and `ingest_work_units`: run state, root path, page work status,
  selected model id, attempts, cancellation/failure/completion markers, and
  progress counters.
- `document_pages` and `document_preview_images`: page status, render metadata,
  source preview image paths, dimensions, and render DPI.
- `document_page_ocr`: raw OCR output, cleaned display text, runtime profile,
  status, attempts, error text, and options metadata.
- `document_regions`: stable region id, file hash, page number, engine/profile,
  label, normalized `TOPLEFT_NORMALIZED_0_999` bounding box, and selected-box
  text content.
- `document_text_region_links`: cleaned text offsets that connect clickable text
  spans to bounding boxes.
- `document_terms`: lowercase token index used by DuckDB-backed search.
- `ingest_diagnostic_events`: persisted diagnostic messages for run progress and
  failures.
- `settings`: persisted workbench settings such as `selected_model_id`, stored
  as JSON values and loaded on startup.

Startup reload reconstructs the in-memory workbench state from DuckDB so prior
documents, text, boxes, previews, and recent runs are visible after restarting
the portable app. `GET /api/documents?q=...` and `GET /api/search?q=...` search
the persisted `document_terms` table first and then fall back to display-name
and cleaned-text matching for partial terms.

## API Contract

`src/uocr-server/openapi/uocr.openapi.json` is the API source of truth. It
covers the currently implemented workbench surface: health/status, trusted
folder selection, model catalog state, model selection, model download
start/cancel/events, ingest start/stop, run snapshots/events, document
lists/details, DuckDB-backed search, regions with selected-box content, text
spans, preview images, recent logs, and settings. Removed or future-only
surfaces are intentionally omitted from the UI and OpenAPI contract until they
work end to end.

The default OCR profile is `experimental-exact-prefill-q4`. The default model
selection is `unlimited-ocr-q4-k-m`, but the model library exposes BF16, Q8,
Q6, Q5, Q4, IQ4, Q3, IQ3, and IQ2 variants from the Sahil GGUF repo. The
selected model id is included in ingest start requests, persisted in DuckDB,
and recorded on each run.

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
