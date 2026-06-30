# Ubuntu 24.04 Portable Workbench

The Linux target is a portable C++/React workbench archive for Ubuntu 24.04.
It runs locally, binds to `127.0.0.1`, hosts the React app from the C++ Drogon
server, and stores state in DuckDB inside the extracted folder.

## Artifacts

| Machine | Artifact | Runtime |
| --- | --- | --- |
| Ubuntu 24.04 x64 | `uocr-workbench-linux-x64-<tag>.tar.gz` | CUDA 13 primary, CPU fallback |
| Ubuntu 24.04 arm64 | `uocr-workbench-linux-arm64-<tag>.tar.gz` | CPU |

The arm64 build is CPU-first because GitHub-hosted `ubuntu-24.04-arm` runners
do not provide CUDA GPU hardware. ARM CUDA can be added later through a
self-hosted runner and a separate runtime label.

## Quick Start

```sh
mkdir -p ~/uocr
cd ~/uocr
tar -xzf ~/Downloads/uocr-workbench-linux-x64-<tag>.tar.gz
cd uocr-workbench-linux-x64-<tag>
./uocr-server.sh
```

For arm64, use the `linux-arm64` archive and folder name instead.

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

Set `HF_TOKEN` before launching if Hugging Face downloads require auth:

```sh
export HF_TOKEN=hf_...
./uocr-server.sh
```

## Runtime Requirements

For `linux-x64`, the CUDA runtime requires:

- NVIDIA driver visible through `nvidia-smi`
- CUDA runtime compatibility with CUDA 13 binaries
- GPU compute capability 7.5 or newer

If CUDA is unavailable, select the CPU runtime in Settings when the CPU fallback
is bundled or installed under `thirdparty/uocr-runtime/linux-x86_64-cpu`.

For `linux-arm64`, the release uses `linux-arm64-cpu`.

## Local Package Build

Install Bun, CMake, Ninja, a C++20 compiler, Python, and vcpkg. Then:

```sh
git submodule update --init --recursive
export VCPKG_ROOT="$HOME/vcpkg"
bash scripts/linux/package-workbench.sh \
  --version v0.0.0-local \
  --runtime-platform linux-x86_64-cuda13 \
  --additional-runtime-platforms linux-x86_64-cpu \
  --package-arch x64 \
  --preset linux-x64-workbench-ci
```

For arm64:

```sh
bash scripts/linux/package-workbench.sh \
  --version v0.0.0-local \
  --runtime-platform linux-arm64-cpu \
  --additional-runtime-platforms "" \
  --package-arch arm64 \
  --preset linux-arm64-workbench-ci
```

Package outputs:

```text
dist/uocr-workbench-linux-x64-<version>.tar.gz
dist/uocr-workbench-linux-x64-<version>.tar.gz.sha256
dist/uocr-workbench-linux-arm64-<version>.tar.gz
dist/uocr-workbench-linux-arm64-<version>.tar.gz.sha256
```

## CI Coverage

GitHub Actions builds Linux x64 and Linux arm64 workbench packages in parallel
with Windows and macOS. Each package job runs frontend checks, builds the C++
server/tests through vcpkg, smokes the extracted archive, verifies logs and
DuckDB creation, and uploads artifacts for the final release publisher.
