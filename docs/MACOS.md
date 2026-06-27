# macOS Metal Quick Start

This guide is for running the portable Unlimited-OCR candidate natively on
macOS without SGLang. It uses:

- `bangonkali/llama.cpp-baidu-unlimited-ocr`, branch
  `uocr-deepseek-ocr-parity`.
- `bangonkali/baidu-unlimited-ocr-portable`, branch `main`.
- GGUF model files from `sahilchachra/Unlimited-OCR-GGUF`.
- A Metal-enabled `llama-uocr-parity` binary.

The default macOS model is `Unlimited-OCR-Q4_K_M.gguf` plus
`mmproj-Unlimited-OCR-F16.gguf`. The GGUF model card marks Q4_K_M as the
recommended size/quality trade-off, and every run needs one language-model GGUF
plus the shared F16 projector.

## Scripted Quick Start

Install prerequisites:

```sh
xcode-select --install
brew install git cmake uv
uv tool install "huggingface-hub[cli]"
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

When doctor has no blocking failures, run the full setup. It initializes
submodules, syncs Python dependencies, downloads the default Q4_K_M GGUF model
plus mmproj into `models/`, builds the native llama.cpp tools with Metal, and
writes runtime environment variables:

```sh
./scripts/mac/setup-build.sh
```

To also download the diagnostic Q5_K_M, Q6_K, and BF16 GGUFs:

```sh
./scripts/mac/setup-build.sh --include-diagnostics
```

The setup script checks:

- `git`
- `cmake`
- `uv`
- `hf`
- Xcode command line tools through `xcode-select`
- Apple clang and macOS SDK through `xcrun`
- Git submodules via `git submodule update --init --recursive`
- Python/Gradio dependencies via `uv sync --frozen`
- Hugging Face authorization via `hf auth whoami` when model downloads are needed
- GGUF downloads into `models/`
- built `llama-uocr-parity`, `llama-mtmd-cli`, and `llama-server`

Useful setup switches:

- `--include-diagnostics`: also download Q5_K_M, Q6_K, and BF16 GGUFs.
- `--force-model-download`: redownload model files even when non-empty local
  files already exist.
- `--skip-python-sync`: skip `uv sync --frozen` if you already synced the project.
- `--skip-model-download`: skip Hugging Face auth and model download.
- `--skip-build`: skip CMake configure/build.
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

- `thirdparty/llama.cpp/build/bin/llama-uocr-parity`
- `models/Unlimited-OCR-Q4_K_M.gguf`
- `models/mmproj-Unlimited-OCR-F16.gguf`

Override with `UOCR_LLAMA_BIN`, `UOCR_MODEL`, and `UOCR_MMPROJ` when testing
another build or model location.

## Known macOS Notes

- This path is intended for local Apple Silicon / Metal inference. Intel Macs
  can still build, but performance and memory headroom should be validated
  separately.
- Quality is still below the SGLang BF16 reference described in the main
  validation summaries.
- BF16 is available as a diagnostic download, but Q4_K_M remains the portable
  default because it is much smaller and the existing demo profile was validated
  around the Q4 path.
