# Trapo Releases

Trapo releases are portable archives that can be extracted anywhere writable and
launched directly. Each archive contains the Rust/Axum server, the compiled
React workbench, PDFium, DuckDB, and the native OCR runtime files for that
platform.

## Artifacts

| Platform | Artifact |
| --- | --- |
| Windows x64 | `trapo-workbench-windows-x64-<tag>.zip` |
| Windows arm64 | `trapo-workbench-windows-arm64-<tag>.zip` |
| macOS arm64 | `trapo-workbench-macos-arm64-<tag>.zip` |
| Ubuntu 24.04 x64 | `trapo-workbench-linux-x64-<tag>.tar.gz` |
| Ubuntu 24.04 arm64 | `trapo-workbench-linux-arm64-<tag>.tar.gz` |

Each artifact has a matching `.sha256` file.

## User Flow

Extract the archive, run the launcher, then open:

```text
http://127.0.0.1:8765/
```

Launchers:

```text
Windows: trapo-server.exe
macOS:   ./trapo-server.sh
Linux:   ./trapo-server.sh
```

Runtime logs and database state are inside the extracted folder:

```text
logs/trapo-server.log
data/trapo.duckdb
```

GGUF model files are downloaded after first launch through the Models page.

## GitHub Actions

`Release workbench` runs on `v*` tags. Platform jobs run in parallel:

- Windows x64 on `windows-2025`
- Windows arm64 on `windows-11-arm`
- macOS arm64 on `macos-15`
- Ubuntu 24.04 x64 on `ubuntu-24.04`
- Ubuntu 24.04 arm64 on `ubuntu-24.04-arm`

Each package job builds the React client, builds the Rust server, packages the
portable archive, smokes the extracted app, verifies `logs/trapo-server.log` and
`data/trapo.duckdb`, and uploads workflow artifacts. A final publish job uploads
all artifacts to one GitHub Release.

`Workbench CI` invokes the unified quality runner for Skylos, React, Rust,
Python, and SCC gates. `Build runtime binaries` publishes native FFI runtime
archives used by Trapo releases.

## Maintainer Flow

Before tagging any release, run the unified quality gate locally and require a
100% pass:

```powershell
uv run python scripts\quality.py --profile ci --parallel
```

Tag releases by incrementing the patch version:

```sh
git tag <next-patch-tag>
git push origin feat/trapo-rust <next-patch-tag>
```

Local Trapo packages are produced by:

```text
scripts/package_trapo_workbench.py
```
