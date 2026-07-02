# Windows Trapo Workbench

Download one of the Windows release assets:

| Platform | Artifact | Launcher |
| --- | --- | --- |
| Windows x64 | `trapo-workbench-windows-x64-<tag>.zip` | `trapo-server.exe` |
| Windows arm64 | `trapo-workbench-windows-arm64-<tag>.zip` | `trapo-server.exe` |

Extract the zip into a writable folder and run `trapo-server.exe`, then open
`http://127.0.0.1:8765/`.

The portable app writes:

```text
logs/trapo-server.log
data/trapo.duckdb
```

The release bundles the React workbench, the Rust Trapo server, PDFium, DuckDB,
and compatible native OCR runtime files. Model GGUF files are downloaded after
first launch from the Models page.

For unattended install on Windows x64:

```powershell
.\scripts\install.ps1 -Version latest
```

Local Trapo packaging is handled by `scripts/package_trapo_workbench.py`.
