# Linux Trapo Workbench

Download the release asset for your runner architecture:

| Platform | Artifact |
| --- | --- |
| Ubuntu 24.04 x64 | `trapo-workbench-linux-x64-<tag>.tar.gz` |
| Ubuntu 24.04 arm64 | `trapo-workbench-linux-arm64-<tag>.tar.gz` |

Extract and launch:

```sh
tar -xzf ~/Downloads/trapo-workbench-linux-x64-<tag>.tar.gz
cd trapo-workbench-linux-x64-<tag>
./trapo-server.sh
```

Open `http://127.0.0.1:8765/`.

The portable app writes `logs/trapo-server.log` and `data/trapo.duckdb` inside
the extracted folder. The package includes PDFium, DuckDB, and compatible native
OCR runtime files. On Linux x64 the CUDA runtime is primary and CPU fallback is
included when available; Linux arm64 is CPU-first.

Local Trapo packaging is handled by `scripts/package_trapo_workbench.py`.
