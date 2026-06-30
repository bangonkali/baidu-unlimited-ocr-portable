# Unlimited-OCR Workbench

This repository now focuses on the Windows portable C++/React workbench. The
Python Gradio app and analysis harness remain in the repo as reference and
validation tools, but they are not the launched product runtime.

## Windows Quick Start

1. Download `uocr-workbench-windows-x64-<tag>.zip` from the GitHub Releases
   page.
2. Extract the zip anywhere writable, for example `C:\uocr\workbench`.
3. Optional: set `HF_TOKEN` in the same terminal before launch if Hugging Face
   requires authenticated downloads.
4. Double-click `uocr-server.exe`, or launch it from that terminal.
5. Open `http://127.0.0.1:8765/` if the browser does not open automatically.
6. In the app, open **Models**, choose a compatible Unlimited-OCR GGUF variant,
   and click **Download** on that model card. The panel shows per-file bytes,
   percent, MiB/s, ETA, auth status, retry, re-download, and cancel.
7. Click **Use** on the downloaded model you want for OCR. The selection is
   persisted in `data\uocr.duckdb` and restored after restart.
8. Click **Choose Folder** to open the Windows folder picker, or paste a folder
   path into the fallback path box.
9. Click **Start Scan**.

Supported inputs are `.pdf`, `.png`, `.jpg`, `.jpeg`, `.bmp`, `.tif`, `.tiff`,
and `.webp`. Multi-page PDFs are rendered in-process by MuPDF embedded in
`uocr-server.exe`; there is no separate `mutool.exe` application in the
portable zip.

## What The Exe Does

`uocr-server.exe` binds only to `127.0.0.1` by default and hosts both the API and
the React app:

```text
http://127.0.0.1:8765/
```

It writes the same operational messages to the terminal and to:

```text
logs\uocr-server.log
```

OCR dashboard state is persisted to DuckDB at:

```text
data\uocr.duckdb
```

The database stores scanned files, rendered pages, OCR text, page status,
bounding boxes, text-to-box links, searchable terms, run state, and diagnostics.
The search box uses that DuckDB state after text has been persisted.

Startup logs include the app root, web root, log path, version, git SHA, and
listening URL. Ingest logs include model loading, folder scan counts, PDF page
rendering, and page OCR progress. Model download logs include auth availability
without printing tokens, metadata checks, current file progress, MiB/s,
verification, cancellation, and failures.

Backend package dependencies come from the vcpkg manifest. The current pinned
baseline resolves Drogon `1.9.13` exactly and OpenSSL `3.6.3` exactly.
Trantor/Drogon use that OpenSSL for TLS, and the model downloader links the
same `OpenSSL::Crypto` target for SHA verification. The portable root therefore
includes the matching vcpkg `libssl*.dll` and `libcrypto*.dll` files. The only
native exceptions are deliberate bundled components: the DuckDB Windows SDK
snapshot and the MuPDF submodule used for the embedded PDF renderer.

The React app also keeps one websocket open at `/api/events`. Backend changes
for model downloads, scan progress, document status, OCR text, bounding boxes,
and logs update the UI without opening per-page event streams.

## Runtime Support

The current Windows portable workbench supports the CUDA runtime label:

```text
windows-x86_64-cuda13
```

CPU inference is not currently packaged or selected by the C++ workbench. The
server status endpoint and UI report CUDA explicitly so this is visible at
runtime. Ubuntu 24.04 CUDA and macOS arm64/Metal are planned next; CPU fallback
would need a separate runtime build and selector.

GGUF model files are downloaded after first launch into `models\` from:

```text
https://huggingface.co/sahilchachra/Unlimited-OCR-GGUF
```

The model library supports BF16, Q8_0, Q6_K, Q5_K_M, Q5_K_S, Q4_K_M, Q4_K_S,
IQ4_NL, IQ4_XS, Q3_K_M, IQ3_M, IQ3_XXS, and IQ2_M. The shared
`mmproj-Unlimited-OCR-F16.gguf` is downloaded once and reused. `Q4_K_M` remains
the recommended default model, while the default OCR profile is
`experimental-exact-prefill-q4`; `best-zero-empty-q4` remains available as the
retry/reference profile. The portable zip bundles native runtime DLLs; it does
not bundle the large GGUF model files.

Authenticated Hugging Face downloads use environment variables only. The server
checks `HF_TOKEN` first, then `HUGGING_FACE_HUB_TOKEN`. Tokens are never shown
in the UI, written to config, or printed to logs.

## Local Build

For a local Windows build from this checkout:

```powershell
git submodule update --init --recursive
.\scripts\windows\build-workbench.ps1
.\scripts\windows\package-workbench.ps1 -Version v0.0.0-local -NoRuntimeDownload
```

The build script compiles MuPDF static libraries, resolves Drogon/OpenSSL/curl
through the repository vcpkg manifest, and builds the React SPA. The package
script validates that
`trantor.dll` imports OpenSSL for TLS and that `uocr-server.exe` imports the
same vcpkg `libcrypto` for SHA verification.

The package script writes:

```text
dist\uocr-workbench-windows-x64-<version>.zip
dist\uocr-workbench-windows-x64-<version>.zip.sha256
```

## More Documentation

- [Windows setup and packaging](docs/WINDOWS.md)
- [C++/Drogon architecture](docs/CXX-DROGON-PORT.md)
- [Runtime binaries](docs/RUNTIME-BINARIES.md)
- [Native FFI ABI](docs/NATIVE-FFI.md)
- [Python reference app and validation harness](docs/PYTHON-REFERENCE-AND-HARNESS.md)
- [Release process](docs/RELEASES.md)
- [Linux follow-up target](docs/LINUX.md)
- [macOS follow-up target](docs/MACOS.md)
