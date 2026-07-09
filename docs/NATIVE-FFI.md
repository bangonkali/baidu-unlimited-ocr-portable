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
used when the host has a compatible GPU and CUDA 13 runtime libraries; otherwise
execution falls back to CPU. Release zips do not ship Python runtimes or NVIDIA
CUDA redistributables (`cudart` / `cublas`); those come from the user machine.
PaddleOCR-VL follows the selected runtime id for generative CUDA (no forced CPU
generative pin).
