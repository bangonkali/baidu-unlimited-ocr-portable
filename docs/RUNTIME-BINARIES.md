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
thirdparty/uocr-runtime/windows-x86_64-cpu/bin/uocr-ffi.dll
thirdparty/libmupdf/copyright
```

The release also bundles Drogon/Trantor/vcpkg DLLs needed by the C++ server and
the generated React build under `web/`. DuckDB is bundled as `duckdb.dll` beside
`uocr-server.exe` and writes the OCR dashboard database to `data/uocr.duckdb`.
Runtime binaries are staged under `thirdparty/uocr-runtime/`; GGUF model files
are downloaded or validated after first launch and are not committed to git.
MuPDF is linked into `uocr-server.exe`; the portable zip must not contain
`mutool.exe`.

Native C++ package dependencies are vcpkg-managed. The current pinned baseline
resolves Drogon `1.9.13` exactly, Trantor `1.5.28`, and OpenSSL `3.6.3`
exactly; Trantor/Drogon use that OpenSSL for TLS, and `uocr-server.exe` uses
the same vcpkg `OpenSSL::Crypto` target for SHA256 verification of downloaded
model files. The release root therefore includes the matching `libssl*.dll` and
`libcrypto*.dll` files. MuPDF is vcpkg `libmupdf` and is statically linked into
`uocr-server.exe`. The Windows DuckDB SDK snapshot is the only dependency
exception because the vcpkg `duckdb` port currently fails on MSVC 19.51 with
`C1083` generated-file errors.
`package-workbench.ps1` and the release workflow inspect dependencies with
`dumpbin` and fail if Trantor is not OpenSSL/TLS-enabled or if the server does
not import `libcrypto` for SHA verification.

Running `uocr-server.exe` appends launch and listener diagnostics to
`logs/uocr-server.log`. Model downloads from the C++ workbench target the
selected Unlimited-OCR GGUF variant plus the shared F16 mmproj file in
`models/`. PDF rendering uses embedded MuPDF at 200 DPI and writes page PNGs
under `cache\rendered-pages\`.

On Windows, the executable requests per-monitor DPI awareness at startup before
the native folder picker is opened. The picker remains native because the server
needs a trusted recursive scan root path; the web UI also keeps the manual path
input for environments where the shell dialog is not desirable.

The built-in model catalog currently covers all compatible GGUF files from
`sahilchachra/Unlimited-OCR-GGUF`:

| Model id | File | Notes |
| --- | --- | --- |
| `unlimited-ocr-bf16` | `Unlimited-OCR-BF16.gguf` | Largest diagnostic/reference model |
| `unlimited-ocr-q8-0` | `Unlimited-OCR-Q8_0.gguf` | High quality, high VRAM |
| `unlimited-ocr-q6-k` | `Unlimited-OCR-Q6_K.gguf` | Medium-high VRAM |
| `unlimited-ocr-q5-k-m` | `Unlimited-OCR-Q5_K_M.gguf` | Balanced higher-quality option |
| `unlimited-ocr-q5-k-s` | `Unlimited-OCR-Q5_K_S.gguf` | Smaller Q5 variant |
| `unlimited-ocr-q4-k-m` | `Unlimited-OCR-Q4_K_M.gguf` | Recommended default model |
| `unlimited-ocr-q4-k-s` | `Unlimited-OCR-Q4_K_S.gguf` | Smaller Q4 variant |
| `unlimited-ocr-iq4-nl` | `Unlimited-OCR-IQ4_NL.gguf` | Edge-tuned I-quant |
| `unlimited-ocr-iq4-xs` | `Unlimited-OCR-IQ4_XS.gguf` | Compact I-quant Q4 |
| `unlimited-ocr-q3-k-m` | `Unlimited-OCR-Q3_K_M.gguf` | Tight-memory option |
| `unlimited-ocr-iq3-m` | `Unlimited-OCR-IQ3_M.gguf` | I-quant 3-bit option |
| `unlimited-ocr-iq3-xxs` | `Unlimited-OCR-IQ3_XXS.gguf` | Very small 3-bit option |
| `unlimited-ocr-iq2-m` | `Unlimited-OCR-IQ2_M.gguf` | Smallest experimental option |

Every model card also requires `mmproj-Unlimited-OCR-F16.gguf`. The UI records
which files are already present, the selected model id, per-file progress,
overall progress, transfer speed, ETA, and the active auth source.

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
| `linux-x86_64-rocm6` | Linux x86_64 | ROCm 6 | `.tar.gz` |
| `linux-x86_64-cpu` | Linux x86_64 | CPU | `.tar.gz` |
| `linux-arm64-cpu` | Linux arm64 | CPU | `.tar.gz` |
| `windows-x86_64-cuda13` | Windows x86_64 | CUDA 13 | `.zip` |
| `windows-x86_64-rocm6` | Windows x86_64 | ROCm 6 | `.zip` |
| `windows-x86_64-cpu` | Windows x86_64 | CPU | `.zip` |

Linux and Windows CUDA labels require `nvidia-smi` to be available when
installing a prebuilt runtime, and the machine must have NVIDIA driver/runtime
libraries compatible with CUDA 13 binaries. If the accelerator probe fails, the
downloader refuses to install a CUDA-labeled binary for that machine.

The C++ workbench runtime selector defaults to CUDA, then ROCm or Metal, then
CPU, but only for runtime directories that are installed and whose hardware
probe passes. The Windows package stages CUDA plus CPU by default; Ubuntu x64
stages CUDA plus CPU when both runtime assets are available; Ubuntu arm64 is
CPU-first. ROCm becomes selectable when a matching runtime is installed and the
AMD probe passes.

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
uocr-workbench-macos-arm64-<version>.zip
uocr-workbench-linux-x64-<version>.tar.gz
uocr-workbench-linux-arm64-<version>.tar.gz
uocr-runtime-macos-arm64-metal-<version>.tar.gz
uocr-runtime-linux-x86_64-cuda13-<version>.tar.gz
uocr-runtime-linux-x86_64-cpu-<version>.tar.gz
uocr-runtime-linux-arm64-cpu-<version>.tar.gz
uocr-runtime-windows-x86_64-cuda13-<version>.zip
uocr-runtime-<platform>-<version>.<ext>.sha256
uocr-runtime-manifest.json
```

The installer reads `uocr-runtime-manifest.json` first and verifies the archive
SHA256 before extracting it. If a release manifest is absent, it can fall back to
the matching archive and `.sha256` assets.

## Building Releases

The workflow `.github/workflows/build-runtime.yml` builds runtime labels in a
matrix and uploads archives as workflow artifacts on manual runs. It publishes
GitHub Release assets when:

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

The active app workflow is `.github/workflows/release-workbench.yml`. It builds
Windows, macOS, Ubuntu x64, and Ubuntu arm64 workbench packages in parallel,
smokes each extracted archive, then publishes all app artifacts from one final
fan-in release job.

Runtime GPU smoke validation should be run on self-hosted GPU runners before
promoting a release as validated; GitHub-hosted standard runners compile the
binaries but do not provide NVIDIA GPUs for inference smoke tests.
