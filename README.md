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

## Ingest Workflow

1. Click the Start OCR icon in the Workbench activity bar.
2. If the selected model is downloaded, Trapo opens the dedicated ingest start
   page immediately.
3. If the selected model is missing locally, Trapo opens the model downloader
   and records a lower-right notification explaining what blocked ingest.
4. Choose a folder on the ingest start page and start the scan.
5. Use the Workbench explorer, preview, text pane, and diagnostics page to
   inspect page images, OCR text, bounding boxes, spans, logs, and timing.

Trapo stores OCR text, bounding boxes, run history, selected runtime/model, and
Workbench UI settings in:

```text
data/trapo.duckdb
```

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

## More Documentation

- [Windows portable app](docs/WINDOWS.md)
- [macOS portable app](docs/MACOS.md)
- [Ubuntu 24.04 portable app](docs/LINUX.md)
- [Workbench flow](docs/WORKBENCH.md)
- [Diagnostics and replay](docs/DIAGNOSTICS.md)
- [Text preview behavior](docs/TEXT-PREVIEW.md)
- [Release process](docs/RELEASES.md)
- [Local Skylos workflow](docs/skylos/README.md)
- [Runtime binaries](docs/RUNTIME-BINARIES.md)
- [Native FFI ABI](docs/NATIVE-FFI.md)
