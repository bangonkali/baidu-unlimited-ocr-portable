# Workbench Releases

The release goal is portable archives that can be extracted anywhere writable
and launched directly. The executable hosts both the Drogon API and the React
app.

## Artifacts

| Platform | Artifact |
| --- | --- |
| Windows x64 | `uocr-workbench-windows-x64-<tag>.zip` |
| macOS arm64 | `uocr-workbench-macos-arm64-<tag>.zip` |
| Ubuntu 24.04 x64 | `uocr-workbench-linux-x64-<tag>.tar.gz` |
| Ubuntu 24.04 arm64 | `uocr-workbench-linux-arm64-<tag>.tar.gz` |

Each artifact has a matching `.sha256` file.

## User Flow

Extract the archive, run the launcher, then open:

```text
http://127.0.0.1:8765/
```

Launchers:

```text
Windows: uocr-server.exe
macOS:   uocr-server.command
Linux:   ./uocr-server.sh
```

Runtime logs and database state are inside the extracted folder:

```text
logs/uocr-server.log
data/uocr.duckdb
```

GGUF model files are downloaded after first launch through the Models page.
The package bundles native runtime binaries, not model weights.

## GitHub Actions

`Release workbench` runs on `v*` tags and manual dispatch. Platform jobs run in
parallel by default:

- Windows x64 on `windows-2025`
- macOS arm64 on `macos-15`
- Ubuntu 24.04 x64 on `ubuntu-24.04`
- Ubuntu 24.04 arm64 on `ubuntu-24.04-arm`

The workflow uses `strategy.fail-fast: false`. Each platform job checks the
frontend, builds the C++ server/tests through vcpkg, packages the portable
archive, smokes the extracted app, verifies `logs/uocr-server.log` and
`data/uocr.duckdb`, and uploads workflow artifacts. A final fan-in `publish`
job downloads all artifacts and uploads them to one GitHub Release.

`Workbench CI` runs frontend quality gates, C++ tests across the same OS family,
and SCC complexity checks. `Build runtime binaries` publishes native FFI runtime
archives, including Linux CPU fallbacks and Linux arm64 CPU.

## Maintainer Flow

Tag releases by incrementing the patch version:

```sh
git tag v0.0.31
git push origin main v0.0.31
```

Manual release dispatch should use the same tag-style version string, for
example `v0.0.31`.

Local package commands:

```powershell
.\scripts\windows\package-workbench.ps1 -Version v0.0.0-local
```

```sh
bash scripts/mac/package-workbench.sh --version v0.0.0-local
bash scripts/linux/package-workbench.sh --version v0.0.0-local
```
