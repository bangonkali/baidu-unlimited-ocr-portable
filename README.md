# Trapo Workbench

Portable local OCR workbench for Windows, macOS, and Ubuntu 24.04. The target
product is the Rust Trapo server executable that hosts the React app itself.

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

The server writes runtime logs inside the extracted folder:

```text
logs/trapo-server.log
```

OCR state, settings, selected model/runtime, workbench pane layout, text spans,
bounding boxes, and scan history persist in:

```text
data/trapo.duckdb
```

## First Run

1. Open **Models** and download a model. `Q4_K_M` is the recommended default.
2. Click **Use** on the downloaded model.
3. Open **Start Ingest**, choose a folder with the picker or paste a path, then
   start the scan.
4. Use the Workbench to inspect page images, text, and bounding boxes. Clicking
   OCR text focuses the matching overlay box; clicking a box focuses the text.

Set `HF_TOKEN` before launching if Hugging Face requires authenticated model
downloads. The token is read from the process environment and is not logged or
stored.

Supported inputs are `.pdf`, `.png`, `.jpg`, `.jpeg`, `.bmp`, `.tif`, `.tiff`,
and `.webp`. Multi-page PDFs are rendered in-process through bundled PDFium.

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
- [Release process](docs/RELEASES.md)
- [Runtime binaries](docs/RUNTIME-BINARIES.md)
- [C++/Drogon architecture](docs/CXX-DROGON-PORT.md)
- [Native FFI ABI](docs/NATIVE-FFI.md)
