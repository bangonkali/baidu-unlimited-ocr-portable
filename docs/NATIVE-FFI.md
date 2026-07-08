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
