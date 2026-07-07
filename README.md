# Trapo

Trapo is a cross-platform RAG Ingest Workbench for PDFs, images, and other
supported local document formats. It is optimized for local setup: one portable
Rust server hosts the React workbench, API, PDF rendering, persistence, runtime
loading, and model download flow.

The name comes from the Filipino word "trapo", meaning rag. The current product
cleans and structures local document content for the larger direction of a
Graph RAG application: OCR is the ingest layer, DuckDB is the local operational
store, and the workbench is the inspection surface for text, layout, spans, and
diagnostics before graph extraction is added.

The repository is now Trapo-first. The supported product path is:

```text
src/trapo-server
src/trapo-client
```

## Quick Start

Download a release from:

```text
https://github.com/bangonkali/baidu-unlimited-ocr-portable/releases
```

Choose the artifact for your machine:

| Platform | Artifact | Launcher |
| --- | --- | --- |
| Windows x64 | `trapo-workbench-windows-x64-<tag>.zip` | `trapo-server.exe` |
| Windows arm64 | `trapo-workbench-windows-arm64-<tag>.zip` | `trapo-server.exe` |
| macOS Apple Silicon | `trapo-workbench-macos-arm64-<tag>.zip` | `./trapo-server.sh` |
| Ubuntu 24.04 x64 | `trapo-workbench-linux-x64-<tag>.tar.gz` | `./trapo-server.sh` |
| Ubuntu 24.04 arm64 | `trapo-workbench-linux-arm64-<tag>.tar.gz` | `./trapo-server.sh` |

Extract the archive into a writable folder, run the launcher, then open:

```text
http://127.0.0.1:8765/
```

## Local Build Releases

Use a local build release when you want to test the current checkout as a
portable app before tagging or publishing a GitHub release. Run these commands
from the repository root:

```powershell
$version = (git describe --tags --dirty --always).Trim()
cargo run -p trapo-server --bin export-openapi -- src/trapo-server/openapi/trapo.openapi.json
uv run python scripts\package_trapo_workbench.py `
  --version $version `
  --platform windows-x64 `
  --runtime-version $version `
  --runtime-platform windows-x86_64-cuda13 `
  --additional-runtime-platforms windows-x86_64-cpu `
  --pdfium-release chromium/7920
```

The packager performs the release build, including the React production bundle,
the Rust `trapo-server` release binary, bundled DuckDB, PDFium, OpenAPI output,
and native OCR runtime files. It writes both an archive and an unpacked staging
directory:

```text
dist/trapo-workbench-windows-x64-<version>.zip
dist/trapo-workbench-windows-x64-<version>.zip.sha256
dist/trapo-workbench-windows-x64-<version>/
```

Launch the staged build directly for local testing:

```powershell
.\dist\trapo-workbench-windows-x64-$version\trapo-server.exe --port 8765 --no-browser
```

On macOS (Apple Silicon), use shell syntax:

```sh
version="$(git describe --tags --dirty --always | tr -d '\n')"
cargo run -p trapo-server --bin export-openapi -- src/trapo-server/openapi/trapo.openapi.json
uv run python scripts/package_trapo_workbench.py \
  --version "$version" \
  --platform macos-arm64 \
  --runtime-version "$version" \
  --runtime-platform macos-arm64-metal \
  --pdfium-release chromium/7920

chmod +x "dist/trapo-workbench-macos-arm64-${version}/trapo-server.sh"
"dist/trapo-workbench-macos-arm64-${version}/trapo-server.sh" --port 8765 --no-browser
```

`--runtime-platform` must be a runtime label from `runtime/platforms.json`
(for macOS Apple Silicon this is `macos-arm64-metal`). The packager now also accepts
`--runtime-platform macos-arm64` as a compatibility alias.

Then open:

```text
http://127.0.0.1:8765/
```

The staged build is the fastest way to test locally because it is already
expanded. The archive and checksum are the portable files to hand to another
machine of the same target platform. Runtime state for the staged build is
written inside that staged directory:

```text
data/trapo.duckdb
logs/trapo-server.log
models/
uploads/
```

Use the same script for other target packages by changing the platform and
runtime arguments:

| Target | `--platform` | `--runtime-platform` | `--additional-runtime-platforms` |
| --- | --- | --- | --- |
| Windows x64 | `windows-x64` | `windows-x86_64-cuda13` | `windows-x86_64-cpu` |
| Windows arm64 | `windows-arm64` | `windows-arm64-cpu` | empty |
| macOS Apple Silicon | `macos-arm64` | `macos-arm64-metal` | empty |
| Ubuntu 24.04 x64 | `linux-x64` | `linux-x86_64-cuda13` | `linux-x86_64-cpu` |
| Ubuntu 24.04 arm64 | `linux-arm64` | `linux-arm64-cpu` | empty |

For release parity, smoke the launched app before handoff:

```powershell
Invoke-RestMethod http://127.0.0.1:8765/api/health
Invoke-RestMethod http://127.0.0.1:8765/api/status
Invoke-RestMethod http://127.0.0.1:8765/api/openapi.json
Invoke-WebRequest http://127.0.0.1:8765/ -UseBasicParsing
```

Finally, run the required quality gate and fix any failures:

```powershell
uv run python scripts\quality.py --profile ci --parallel
```

## Ingest Workflow

1. Click the Start OCR icon in the Workbench activity bar.
2. If the selected model is downloaded, Trapo opens the dedicated ingest start
   page immediately.
3. If the selected model is missing locally, Trapo opens the model library and
   records a lower-right notification explaining what blocked ingest.
4. Choose a folder on the ingest start page and start the scan.
5. Use the Workbench explorer, preview, text pane, and diagnostics page to
   inspect page images, OCR text, bounding boxes, spans, logs, and timing.

Trapo stores OCR text, bounding boxes, run history, selected runtime/model, and
Workbench UI settings in:

```text
data/trapo.duckdb
```

Download lifecycle events are stored there as well. The active Download Manager
shows only queued or in-progress files; missing files stay in the model library
as a neutral, restorable state and can be downloaded again.

Page numbers are numeric throughout the stack. DuckDB stores page counts and
page numbers as `INTEGER`, the OpenAPI schema exposes page fields as integers,
and the React workbench treats generated TypeScript page fields as `number`.
Explorer labels such as `Page 10` are display text only, so page rows sort by
actual page number instead of string order.

Runtime logs are written to:

```text
logs/trapo-server.log
```

## Supported Inputs

Trapo currently ingests `.pdf`, `.png`, `.jpg`, `.jpeg`, `.bmp`, `.tif`,
`.tiff`, and `.webp`. Multi-page PDFs are rendered in-process through bundled
PDFium before OCR. The resulting page text and layout boxes are designed for
downstream retrieval-augmented generation pipelines.

## Live Text Preview

During OCR, the text preview intentionally shows only pages that have started or
are already complete. Queued future pages are hidden, so auto-follow stays on
the page that is currently streaming instead of jumping to the last placeholder
page in a large PDF.

The backend marks each page `running` as OCR starts and omits queued pages from
document text payloads. The React workbench also filters stale full-page payloads
as a defensive guard.

Live OCR stream events are persisted independently before they are broadcast to
the browser. On refresh, the workbench can replay historical page events from
DuckDB and then continue applying new realtime events, so the same UI can be
rebuilt while OCR is running or after a page has completed.

Starting a new ingest also seeds the workbench from the accepted run snapshot
before routing to `/workbench`. The first discovered document is selected with
auto-follow enabled, and any missed page stream events are recovered from the
DuckDB replay log.

Auto-follow is route-aware. Manual document or region focus writes
`follow=false` into the workbench route, while the Preview pane toggle writes
`follow=true` when the user turns following back on.

Detected regions are represented by compact text anchors. A region's content
scope runs from its anchor to the next anchor, so PDF bounding boxes, details,
and text-preview focus use the same boundary. OCR HTML tables render as tables,
and image-like regions can embed cropped local snippets in the Markdown preview.
See [Text Preview](docs/TEXT-PREVIEW.md).

## Runtime Notes

- Windows x64 bundles CUDA 13 plus CPU fallback.
- macOS arm64 bundles the Metal runtime.
- Ubuntu 24.04 x64 targets CUDA 13 and bundles CPU fallback when available.
- Ubuntu 24.04 arm64 is CPU-first on GitHub-hosted runners.

GGUF model files are downloaded after first launch into `models/` from:

```text
https://huggingface.co/sahilchachra/Unlimited-OCR-GGUF
```

The release archive contains the native runtime binaries and the React app, but
not the large GGUF model files.

## Quality Gate

Every task must finish with a 100% passing unified gate:

```powershell
uv run python scripts\quality.py --profile ci --parallel
```

The report is written to `.logs/quality/quality-report.md`. See
[Quality gates](docs/QUALITY.md) for per-gate details and CI output flags.

## More Documentation

- [Windows portable app](docs/WINDOWS.md)
- [macOS portable app](docs/MACOS.md)
- [Ubuntu 24.04 portable app](docs/LINUX.md)
- [Workbench flow](docs/WORKBENCH.md)
- [OCR data model](docs/OCR-DATA-MODEL.md)
- [Diagnostics and replay](docs/DIAGNOSTICS.md)
- [Text preview behavior](docs/TEXT-PREVIEW.md)
- [Release process](docs/RELEASES.md)
- [Quality gates](docs/QUALITY.md)
- [Local Skylos workflow](docs/skylos/README.md)
- [Runtime binaries](docs/RUNTIME-BINARIES.md)
- [Native FFI ABI](docs/NATIVE-FFI.md)
