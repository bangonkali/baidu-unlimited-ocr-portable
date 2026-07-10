# Native FFI ABI

Trapo loads local OCR runtimes through packaged native FFI libraries. The legacy
Unlimited OCR path still uses `uocr-ffi`; PP-OCRv6 and PaddleOCR-VL use the
shared `trapo-ocr-ffi` library in process.

The Rust adapter lives in:

```text
src/trapo-server/src/app/ocr_engines/common/native_ocr_ffi.rs
```

The native implementation is Trapo-owned source under:

```text
src/trapo-ocr-native
```

The exported C ABI uses `trapo_ocr_*` symbols.

At startup and during release smoke tests, Trapo validates the ABI version and
required symbols before creating an OCR session. Runtime package CI also checks
the exported ABI with:

```text
scripts/test_ctypes_runtime.py --abi-only
```

macOS packages additionally validate that runtime dylibs use portable loader
paths and do not depend on Homebrew-only absolute library paths.

`trapo-ocr-ffi` receives runtime backend preferences from the selected runtime
id. CUDA runtimes request ONNX Runtime CUDA and llama.cpp CUDA; macOS Metal
runtimes request ONNX CoreML with automatic generative backend selection; ROCm
labels currently request automatic native backend selection until the native ABI
has a validated ROCm-specific backend; CPU runtimes force CPU-only execution.

## CUDA 13 FFI builds (GPU-less CI safe)

For `*-cuda13` platforms, `scripts/build_trapo_ocr_ffi.py` enables the nested
llama.cpp CUDA backend (`TRAPO_LLAMA_ENABLE_CUDA=1`) and sets portable
`TRAPO_CUDA_ARCHITECTURES` (never `native`) so GitHub-hosted runners without a
GPU can still produce CUDA-capable `trapo-ocr-ffi` binaries. CPU, arm64, and
Metal platforms keep all llama hardware backends off.

ONNX Runtime for the FFI still links against the CPU ORT core for load
portability; cuda13 packages additionally stage prebuilt
`onnxruntime_providers_cuda*` libraries. At runtime, CUDA EP and llama CUDA are
used when the host has a compatible GPU and driver; otherwise execution falls
back to CPU. CUDA 13 packages bundle the redistributable CUDA runtime libraries
used by ONNX Runtime and llama.cpp (`cudart`, cuBLAS, cuFFT, cuRAND, NVRTC, and
NVJitLink) plus the cuDNN 9 runtime. The host still supplies the NVIDIA driver
(`nvcuda.dll` / `libcuda.so.1`). Release packages do not ship Python runtimes.
PaddleOCR-VL follows the selected runtime id for generative CUDA (no forced CPU
generative pin).

`runtime/nvidia-redist.json` pins the cuDNN wheel and declares every required
runtime filename. `scripts/nvidia_redist_staging.py` copies CUDA libraries from
the toolkit used for the build and downloads the checksum-pinned cuDNN runtime.
On Windows it also stages `zlibwapi.dll` (cuDNN 9 still resolves this by
basename) plus the NVIDIA CUDA/cuDNN notices and zlib readme. These NVIDIA
files are redistributed only inside the Trapo application under the
[CUDA Toolkit EULA](https://docs.nvidia.com/cuda/eula/index.html) and
[cuDNN EULA](https://docs.nvidia.com/deeplearning/cudnn/latest/reference/eula.html);
do not publish them as a stand-alone SDK or relicense them under Trapo's
open-source license.

Windows packages also stage the app-local MSVC runtime DLLs from
`VCToolsRedistDir` through `scripts/windows_runtime_staging.py`; users do not
need a separate Visual C++ Redistributable installation.

`trapo-server` prepends packaged `thirdparty/uocr-runtime/*/bin` directories to
`PATH` at startup (CUDA bins first) so starting `trapo-server.exe` without
`trapo-server.cmd` still resolves cuDNN sibling DLLs. Without that search path,
cuDNN prints `Invalid handle. Cannot load symbol cudnnCreate` and the ORT CUDA
EP fails to initialize.

PP-OCRv6 on `*-cuda13` requests the ONNX Runtime CUDA execution provider for its
detector/recognizer sessions (same EP append path as Document Markdown), with
automatic CPU fallback if the CUDA provider or device is unavailable. Catalog
`accelerator: cuda` only means the cuda13 runtime variant is selected; confirm
live EP usage via native `runtimeSummary` (for example `onnxruntime cuda`) or
GPU utilization during a PP-OCR job.
