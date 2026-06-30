# macOS Apple Silicon Portable Workbench

The macOS target is a portable Apple Silicon archive that bundles the C++
Drogon server, the React app, vcpkg MuPDF/DuckDB dependencies, and the
`macos-arm64-metal` native runtime. GGUF model files are downloaded after first
launch.

## Quick Start

Download:

```text
uocr-workbench-macos-arm64-<tag>.zip
```

Extract it, then run:

```sh
./uocr-server.command
```

Open:

```text
http://127.0.0.1:8765/
```

Logs and state:

```text
logs/uocr-server.log
data/uocr.duckdb
models/
cache/
```

Optional Hugging Face auth:

```sh
export HF_TOKEN=hf_...
./uocr-server.command
```

If macOS quarantine blocks launch after downloading from a browser, clear the
archive/folder quarantine bit before running:

```sh
xattr -dr com.apple.quarantine ./uocr-workbench-macos-arm64-<tag>
```

## Runtime

The bundled runtime label is:

```text
macos-arm64-metal
```

Intel macOS is not a published portable artifact. Build locally only for
experimentation and validate performance separately.

## Local Package Build

Install Xcode command line tools, CMake, Ninja, Bun, Python, and vcpkg. Then:

```sh
git submodule update --init --recursive
export VCPKG_ROOT="$HOME/vcpkg"
bash scripts/mac/package-workbench.sh \
  --version v0.0.0-local \
  --runtime-version latest \
  --runtime-platform macos-arm64-metal \
  --preset macos-arm64-workbench-ci
```

Package outputs:

```text
dist/uocr-workbench-macos-arm64-<version>.zip
dist/uocr-workbench-macos-arm64-<version>.zip.sha256
```

## CI Coverage

GitHub Actions builds the macOS package in parallel with Windows and Linux.
The job runs frontend checks, builds the C++ server/tests through vcpkg, smokes
the extracted archive on `macos-15`, verifies logs and DuckDB creation, and
uploads artifacts for the final release publisher.
