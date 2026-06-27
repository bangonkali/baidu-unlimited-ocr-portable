# macOS Metal Quick Start

This guide is for running the portable Unlimited-OCR candidate natively on
macOS without SGLang. It uses:

- `bangonkali/llama.cpp-baidu-unlimited-ocr`, branch
  `uocr-deepseek-ocr-parity`.
- `bangonkali/baidu-unlimited-ocr-portable`, branch `main`.
- GGUF model files from `sahilchachra/Unlimited-OCR-GGUF`.
- A Metal-enabled `llama-uocr-parity` binary, downloaded from GitHub Releases
  by default or built locally on request.

The default macOS model is `Unlimited-OCR-Q4_K_M.gguf` plus
`mmproj-Unlimited-OCR-F16.gguf`. The GGUF model card marks Q4_K_M as the
recommended size/quality trade-off, and every run needs one language-model GGUF
plus the shared F16 projector.

## Scripted Quick Start

Install prerequisites:

```sh
brew install git uv
uv tool install "huggingface-hub[cli]"
```

For local source builds, also install Xcode command line tools and CMake:

```sh
xcode-select --install
brew install cmake
```

If you already have the Hugging Face CLI from another source, verify it is
available as `hf`:

```sh
hf --version
```

Authenticate before the first model download:

```sh
hf auth whoami || hf auth login
```

Clone the portable repo recursively and run the doctor preflight first. Doctor
does not download models, build native code, or write generated env files; it
checks the repo, submodule, command-line tools, Xcode/clang, lockfile, and
local model/build cache status.

```sh
mkdir -p ~/uocr
cd ~/uocr

git clone --recursive git@github.com:bangonkali/baidu-unlimited-ocr-portable.git \
  unlimited-ocr-portable

cd ~/uocr/unlimited-ocr-portable

./scripts/mac/setup-build.sh --doctor
```

When doctor has no blocking failures, run the full setup. It syncs Python
dependencies, downloads the default Q4_K_M GGUF model plus mmproj into
`models/`, installs the prebuilt `macos-arm64-metal` runtime from GitHub
Releases, and writes runtime environment variables:

```sh
./scripts/mac/setup-build.sh
```

To compile the Metal runtime locally instead of downloading it:

```sh
./scripts/mac/setup-build.sh --runtime-source build
```

To try download first and compile only if no release asset is available:

```sh
./scripts/mac/setup-build.sh --runtime-source auto
```

To also download the diagnostic Q5_K_M, Q6_K, and BF16 GGUFs:

```sh
./scripts/mac/setup-build.sh --include-diagnostics
```

The setup script checks:

- `git`
- `uv`
- `hf`
- Python/Gradio dependencies via `uv sync --frozen`
- Hugging Face authorization via `hf auth whoami` when model downloads are needed
- GGUF downloads into `models/`
- downloaded or built `llama-uocr-parity`, `llama-mtmd-cli`, and `llama-server`

When `--runtime-source build` is used, it also checks `cmake`, Xcode command
line tools through `xcode-select`, Apple clang and the macOS SDK through
`xcrun`, and the `llama.cpp` submodule.

Useful setup switches:

- `--include-diagnostics`: also download Q5_K_M, Q6_K, and BF16 GGUFs.
- `--force-model-download`: redownload model files even when non-empty local
  files already exist.
- `--runtime-source download|build|auto`: choose prebuilt runtime download,
  local compilation, or download-with-build-fallback. Default: `download`.
- `--runtime-version TAG`: download a specific GitHub Release tag instead of
  the latest release.
- `--force-runtime-download`: redownload and reinstall the prebuilt runtime.
- `--skip-python-sync`: skip `uv sync --frozen` if you already synced the project.
- `--skip-model-download`: skip Hugging Face auth and model download.
- `--skip-build`: skip CMake configure/build when using `--runtime-source build`
  or an `auto` fallback build.
- `--generator Ninja`: choose a CMake generator.

The script writes:

```text
uocr-runtime-env.sh
```

Run a smoke test after copying a test image into `dataset/`:

```sh
./scripts/mac/run-demo.sh \
  --smoke \
  --image dataset/sc-02.png \
  --max-tokens 64
```

Launch the Gradio demo:

```sh
./scripts/mac/run-demo.sh --host 127.0.0.1 --port 7861
```

Open:

```text
http://127.0.0.1:7861
```

The UI defaults to the persistent `ffi` runtime backend. It starts
`llama-server` once, keeps the model and mmproj resident, and processes all PDF
pages through that session. Use the runtime selector in the header, or
`baidu-uocr-client --smoke --runtime-backend executable`, to force the legacy
per-request executable path.

## Manual Build Notes

The scripted build uses the same CMake targets as the Linux and Windows docs:

```sh
cmake -B thirdparty/llama.cpp/build \
  -S thirdparty/llama.cpp \
  -DGGML_METAL=ON \
  -DCMAKE_BUILD_TYPE=Release

cmake --build thirdparty/llama.cpp/build \
  --config Release \
  --target llama-mtmd-cli llama-uocr-parity llama-server \
  --parallel
```

The upstream llama.cpp build docs enable Metal by default on macOS, but the
portable script passes `-DGGML_METAL=ON` explicitly so the build intent is clear.

Default runtime paths:

- `thirdparty/uocr-runtime/macos-arm64-metal/bin/llama-uocr-parity`
- `models/Unlimited-OCR-Q4_K_M.gguf`
- `models/mmproj-Unlimited-OCR-F16.gguf`

Override with `UOCR_LLAMA_BIN`, `UOCR_MODEL`, and `UOCR_MMPROJ` when testing
another build or model location.

## Known macOS Notes

- This path is intended for local Apple Silicon / Metal inference. Intel Macs
  are not a prebuilt runtime target; use `--runtime-source build` if you need to
  experiment there, and validate performance separately.
- Quality is still below the SGLang BF16 reference described in the main
  validation summaries.
- BF16 is available as a diagnostic download, but Q4_K_M remains the portable
  default because it is much smaller and the existing demo profile was validated
  around the Q4 path.
