# Runtime Binaries

Trapo packages consume native OCR runtime archives from:

```text
thirdparty/uocr-runtime/<platform>
```

The runtime asset names still use the `uocr-runtime-*` prefix for compatibility.
The archive now carries two native OCR ABIs: existing `uocr-ffi` for Unlimited
OCR and shared `trapo-ocr-ffi` for PP-OCRv6 and PaddleOCR-VL.

The `Build runtime binaries` workflow creates and publishes runtime archives for
the supported platforms. `Release workbench` downloads those assets before
running:

```text
scripts/package_trapo_workbench.py
```

Every supported runtime archive must include the native engine command surface:

```text
llama-mtmd-cli
trapo-ocr-ffi
trapo-tesseract-rs-runner
```

Windows runner archives use `.exe` names and Windows libraries use `.dll`.
PP-OCRv6 and PaddleOCR-VL use `trapo-ocr-ffi` in process with their packaged
assets. Remaining GGUF document-understanding engines continue to use
`llama-mtmd-cli` until they are migrated. Tesseract still uses a Trapo runner
wrapper around the packaged Tesseract assets until its FFI migration is complete.

Every supported runtime archive must also include the engine payload directories:

```text
bin/trapo-ocr-ffi(.dll/.so/.dylib)
ppocrv6/models/manifest.json
paddleocr_vl_1_6/manifest.json
paddleocr_vl_1_6/layout_detection/inference.onnx
tesseract/bin/tesseract(.exe)
tesseract/tessdata/eng.traineddata
```

`scripts/install_ppocrv6_runtime.py` reuses
`C:\Users\Bangonkali\Desktop\Projects\embedded-ocr\assets\models\ppocrv6_medium_full`
when it is available, then falls back to the pinned Hugging Face ONNX model
files from that manifest. It does not create `.venv`, PyInstaller output, or
Python fallback assets. `scripts/build_trapo_ocr_ffi.py` builds and stages the
shared native OCR FFI from the Trapo-owned `src/trapo-ocr-native` source tree.
`scripts/install_tesseract_runtime.py` can stage an installed
Tesseract binary or build from the `thirdparty/tesseract` submodule, then
installs English tessdata into the runtime payload.

Supported runtime targets:

```text
macos-arm64-metal
linux-x86_64-cuda13
linux-x86_64-cpu
linux-arm64-cpu
windows-x86_64-cuda13
windows-x86_64-cpu
windows-arm64-cpu
```

Planned ROCm targets stay marked as planned in `runtime/platforms.json` until
they are validated on appropriate hardware and promoted into the Actions matrix:

```text
linux-x86_64-rocm6
windows-x86_64-rocm6
```

Relevant runtime tooling:

```text
scripts/package_runtime.py
scripts/install_runtime.py
scripts/test_ctypes_runtime.py
scripts/package_trapo_workbench.py
scripts/runtime_engine_guard.py
scripts/build_trapo_ocr_ffi.py
scripts/install_ppocrv6_runtime.py
scripts/install_tesseract_runtime.py
```

Runtime guard coverage:

```text
python scripts/runtime_engine_guard.py manifest
python scripts/runtime_engine_guard.py smoke-runners --platform <target> --build-dir thirdparty/llama.cpp/build
python scripts/runtime_engine_guard.py packaged-runtime --platform <target> --version <version> --dist-dir dist
```

The unified quality gate also exposes this as:

```text
uv run python scripts/quality.py --profile ci --only runtime
```

Release archives are native-binary portable: no Python OCR engines, `.venv`, or
shipped NVIDIA CUDA redistributables. Models are downloaded at runtime through
the app. cuda13 packages include CUDA-capable llama/`uocr-ffi` and
`trapo-ocr-ffi` builds plus staged ONNX Runtime CUDA provider libraries; hosts
without a GPU fall back to CPU.

The Trapo portable archives include:

```text
trapo-workbench-windows-x64-<version>.zip
trapo-workbench-windows-arm64-<version>.zip
trapo-workbench-macos-arm64-<version>.zip
trapo-workbench-linux-x64-<version>.tar.gz
trapo-workbench-linux-arm64-<version>.tar.gz
```
