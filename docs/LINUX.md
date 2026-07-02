# Linux Trapo

Download the Ubuntu 24.04 release asset for your architecture:

| Platform | Artifact |
| --- | --- |
| Ubuntu 24.04 x64 | `trapo-workbench-linux-x64-<tag>.tar.gz` |
| Ubuntu 24.04 arm64 | `trapo-workbench-linux-arm64-<tag>.tar.gz` |

Extract the archive and launch:

```sh
tar -xzf ~/Downloads/trapo-workbench-linux-x64-<tag>.tar.gz
cd trapo-workbench-linux-x64-<tag>
./trapo-server.sh
```

Then open:

```text
http://127.0.0.1:8765/
```

The archive bundles the React workbench, Rust server, PDFium, DuckDB, and native
OCR runtime files. Ubuntu x64 targets CUDA 13 with CPU fallback when available;
Ubuntu arm64 is CPU-first on GitHub-hosted runners.
