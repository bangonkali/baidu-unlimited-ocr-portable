# Runtime Binaries

Trapo packages consume native OCR runtime archives from:

```text
thirdparty/uocr-runtime/<platform>
```

The runtime asset names still use the `uocr-runtime-*` prefix because the
underlying native ABI is named `uocr-ffi`. That ABI is an internal runtime
boundary; the product, server, client, and release artifacts are Trapo.

The `Build runtime binaries` workflow creates and publishes runtime archives for
the supported platforms. `Release workbench` downloads those assets before
running:

```text
scripts/package_trapo_workbench.py
```

Every supported runtime archive must include the native engine command surface:

```text
llama-mtmd-cli
trapo-tesseract-rs-runner
trapo-pp-ocrv6-runner
```

Windows archives use `.exe` names. The GGUF engines share `llama-mtmd-cli` and
select model/mmproj files through the server engine registry. Tesseract and
PP-OCRv6 use the Trapo runner wrappers so the server talks to a stable process
contract instead of ad hoc local commands.

Every supported runtime archive must also include the engine payload directories:

```text
ppocrv6/trapo_ppocrv6_engine.py
ppocrv6/bin/trapo_ppocrv6_engine(.exe)
ppocrv6/models/manifest.json
tesseract/bin/tesseract(.exe)
tesseract/tessdata/eng.traineddata
```

`scripts/install_ppocrv6_runtime.py` reuses
`C:\Users\Bangonkali\Desktop\Projects\embedded-ocr\assets\models\ppocrv6_medium_full`
when it is available, then falls back to the pinned Hugging Face ONNX model
files from that manifest. `scripts/install_tesseract_runtime.py` can stage an
installed Tesseract binary or build from the `thirdparty/tesseract` submodule,
then installs English tessdata into the runtime payload.

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

The Trapo portable archives include:

```text
trapo-workbench-windows-x64-<version>.zip
trapo-workbench-windows-arm64-<version>.zip
trapo-workbench-macos-arm64-<version>.zip
trapo-workbench-linux-x64-<version>.tar.gz
trapo-workbench-linux-arm64-<version>.tar.gz
```
