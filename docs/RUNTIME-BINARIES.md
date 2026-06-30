# Runtime Binary Releases

The setup scripts download prebuilt native runtime binaries from GitHub Releases
by default. Source builds are still available when a user wants to compile
locally or when a release asset is not available.

GGUF model files are not bundled in GitHub Release runtime archives. The setup
scripts continue to download models from Hugging Face into `models/`.

## C++ App Release Layout

The Windows C++ workbench GitHub Release asset is:

```text
uocr-workbench-windows-x64-<tag>.zip
```

Extracting it produces a folder that contains:

```text
uocr-server.exe
uocr-server.cmd
web/
models/
data/
cache/
logs/
config/
uploads/
openapi/uocr.openapi.json
duckdb.dll
thirdparty/uocr-runtime/windows-x86_64-cuda13/bin/uocr-ffi.dll
thirdparty/mupdf/COPYING
```

The release also bundles Drogon/Trantor/vcpkg DLLs needed by the C++ server and
the generated React build under `web/`. DuckDB is bundled as `duckdb.dll` beside
`uocr-server.exe` and writes the OCR dashboard database to `data/uocr.duckdb`.
Runtime binaries are staged under `thirdparty/uocr-runtime/`; GGUF model files
are downloaded or validated after first launch and are not committed to git.
MuPDF is linked into `uocr-server.exe`; the portable zip must not contain
`mutool.exe`.

Running `uocr-server.exe` appends launch and listener diagnostics to
`logs/uocr-server.log`. Model downloads from the C++ workbench currently target
the Unlimited-OCR Q4_K_M model and F16 mmproj files in `models/`. PDF rendering
uses embedded MuPDF at 200 DPI and writes page PNGs under
`cache\rendered-pages\`.

The C++ workbench downloads Hugging Face files through its embedded libcurl
client, not Python or the `hf` CLI. It reads `HF_TOKEN`, then
`HUGGING_FACE_HUB_TOKEN`, from the server process environment. The Models UI and
logs report whether env auth is active, but token values are never printed or
stored. Progress is tracked per file and overall with bytes, percent, MiB/s,
ETA, cancel, retry, and re-download support.

Build the release zip locally:

```powershell
.\scripts\windows\package-workbench.ps1 -Version v0.0.9
```

## Supported Platform Labels

The exact supported runtime labels are defined in `runtime/platforms.json`:

| Label | OS / arch | Backend | Archive |
| --- | --- | --- | --- |
| `macos-arm64-metal` | macOS arm64 | Metal | `.tar.gz` |
| `linux-x86_64-cuda13` | Linux x86_64 | CUDA 13 | `.tar.gz` |
| `windows-x86_64-cuda13` | Windows x86_64 | CUDA 13 | `.zip` |

Linux and Windows CUDA labels require `nvidia-smi` to be available when
installing a prebuilt runtime, and the machine must have NVIDIA driver/runtime
libraries compatible with CUDA 13 binaries. If the accelerator probe fails, the
downloader refuses to install a CUDA-labeled binary for that machine.

The Windows C++ workbench currently selects only `windows-x86_64-cuda13`.
There is no packaged CPU runtime selector in the workbench yet.

CUDA 13 release binaries target compute capability 7.5 or newer and explicitly
include Blackwell RTX 5090 support. NVIDIA's RTX 5090 specs list CUDA
capability `12.0`, which maps to `sm_120`; this runtime builds that path as
`120a-real`.

## Setup Modes

macOS and Linux:

```sh
./scripts/mac/setup-build.sh --runtime-source download
./scripts/linux/setup-build.sh --runtime-source download
```

Windows:

```powershell
.\scripts\windows\setup-build.ps1 -RuntimeSource download
```

`download` is the default and installs the exact supported release asset into:

```text
thirdparty/uocr-runtime/<platform-label>/
```

Other modes:

- `build`: compile `thirdparty/llama.cpp` locally with the platform backend.
- `auto`: try release download first, then compile locally if download fails.

Useful download options:

- mac/Linux: `--runtime-version TAG`, `--runtime-repo OWNER/REPO`,
  `--force-runtime-download`, `--skip-runtime-download`
- Windows: `-RuntimeVersion TAG`, `-RuntimeRepo OWNER/REPO`,
  `-ForceRuntimeDownload`, `-SkipRuntimeDownload`

The scripts write `uocr-runtime-env.sh` or `uocr-runtime-env.ps1` with
`UOCR_FFI_LIB`, `UOCR_LLAMA_BIN`, model paths, and runtime metadata.

The runtime archives include `libuocr-ffi`/`uocr-ffi.dll`, `llama-uocr-parity`,
`llama-mtmd-cli`, and `llama-server`. The C++ workbench uses the `uocr-ffi`
shared library through `UnlimitedOcrFfiEngine`. The Python reference app
defaults to the persistent `ffi` backend through `ctypes`; its `server` and
`executable` backends remain comparison paths.

The native ABI contract is documented in [NATIVE-FFI.md](NATIVE-FFI.md).

## Release Assets

Runtime release assets use these names:

```text
uocr-workbench-windows-x64-<version>.zip
uocr-workbench-windows-x64-<version>.zip.sha256
uocr-runtime-macos-arm64-metal-<version>.tar.gz
uocr-runtime-linux-x86_64-cuda13-<version>.tar.gz
uocr-runtime-windows-x86_64-cuda13-<version>.zip
uocr-runtime-<platform>-<version>.<ext>.sha256
uocr-runtime-manifest.json
```

The installer reads `uocr-runtime-manifest.json` first and verifies the archive
SHA256 before extracting it. If a release manifest is absent, it can fall back to
the matching archive and `.sha256` assets.

## Building Releases

The workflow `.github/workflows/build-runtime.yml` is intentionally disabled
while the Windows portable zip is being stabilized. Re-enable it when runtime
matrix builds are back in scope. When enabled, it builds and packages all three
runtime labels and uploads archives as workflow artifacts on every manual run.

It publishes GitHub Release assets when:

- a `v*` tag is pushed, or
- `workflow_dispatch` is run with `publish=true` and `version` set.

CUDA builds install the CUDA toolkit in CI and compile the CUDA backend without
requiring a GPU on the hosted runner. The workflow installs CMake 4.2.1 or
newer for Blackwell architecture parsing and explicitly sets
`CMAKE_CUDA_ARCHITECTURES` to:

```text
75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real
```

This avoids `native` GPU probing on GPU-less hosted runners and keeps the
archive label tied to CUDA 13 rather than to one CI machine. The `120a-real`
entry is the RTX 5090 / `sm_120` path. CUDA matrix builds also cap CMake
parallelism to reduce hosted-runner memory pressure.

The active workflow is `.github/workflows/release-workbench.yml`. It builds the
Windows C++/React workbench, downloads the selected Windows CUDA runtime
archive, packages `uocr-workbench-windows-x64-<tag>.zip`, smokes the extracted
executable, and uploads the zip plus checksum to the same GitHub Release.

Runtime GPU smoke validation should be run on self-hosted GPU runners before
promoting a release as validated; GitHub-hosted standard runners compile the
binaries but do not provide NVIDIA GPUs for inference smoke tests.
