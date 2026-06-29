# Unlimited-OCR Workbench

This repository now focuses on the Windows portable C++/React workbench. The
Python Gradio app and analysis harness remain in the repo as reference and
validation tools, but they are not the launched product runtime.

## Windows Quick Start

1. Download `uocr-workbench-windows-x64-<tag>.zip` from the GitHub Releases
   page.
2. Extract the zip anywhere writable, for example `C:\uocr\workbench`.
3. Double-click `uocr-server.exe`.
4. Open `http://127.0.0.1:8765/` if the browser does not open automatically.
5. In the app, open **Models** and click **Download model**.
6. Click **Choose Folder** to open the Windows folder picker, or paste a folder
   path into the fallback path box.
7. Click **Start Scan**.

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

Startup logs include the app root, web root, log path, version, git SHA, and
listening URL. Ingest logs include model loading, folder scan counts, PDF page
rendering, and page OCR progress.

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

The portable zip bundles native runtime DLLs; it does not bundle the large GGUF
model files.

## Local Build

For a local Windows build from this checkout:

```powershell
git submodule update --init --recursive
.\scripts\windows\build-workbench.ps1
.\scripts\windows\package-workbench.ps1 -Version v0.0.0-local -NoRuntimeDownload
```

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
