# Workbench Releases

The release goal is portable archives that can be extracted anywhere writable
and launched directly. The executable hosts both the Trapo Rust/Axum API and
the React app.

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
The package bundles native runtime binaries, not model weights.

## GitHub Actions

`Release workbench` runs on `v*` tags. Platform jobs run in parallel by
default:

- Windows x64 on `windows-2025`
- Windows arm64 on `windows-11-arm`
- macOS arm64 on `macos-15`
- Ubuntu 24.04 x64 on `ubuntu-24.04`
- Ubuntu 24.04 arm64 on `ubuntu-24.04-arm`

The workflow uses `strategy.fail-fast: false`. Each platform job checks the
Trapo frontend, builds the Rust server, packages the portable archive, smokes
the extracted app, verifies `logs/trapo-server.log` and `data/trapo.duckdb`,
and uploads workflow artifacts. A final fan-in `publish` job downloads all
artifacts and uploads them to one GitHub Release.

`Workbench CI` runs Trapo frontend quality gates, Rust tests across the same OS
family, Python release-tool tests, and SCC complexity checks. `Build runtime
binaries` publishes native FFI runtime archives, including Linux CPU fallbacks
and Linux arm64 CPU.

## Maintainer Flow

Tag releases by incrementing the patch version:

```sh
git tag v0.0.31
git push origin feat/trapo-rust v0.0.31
```

Local Trapo packages are produced by `scripts/package_trapo_workbench.py`.
