# Portable Client

This demo runs the practical portable Unlimited-OCR candidate through the
patched native llama.cpp runtime. It uses a custom browser UI with live input
annotations and does not use SGLang, PyTorch, Transformers, or the Baidu custom
SGLang wheel.

Default profile:

- `llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full`
- Q4_K_M GGUF
- DeepSeek-OCR gundam preprocessing
- forced prompt EOS
- SGLang-style no-repeat ngram controls
- core reference-SWA masking with a 128-token Unlimited-OCR fallback

The UI also exposes the exact-prefill/no-image-end R-SWA profile as an
experimental option. That profile had higher average similarity in the full
matrix, but it produced empty rows and is not the default.

The default runtime backend is `ffi`, a persistent native session loaded through
`ctypes` from `libuocr-ffi`/`uocr-ffi.dll`. It keeps the model and mmproj
resident across image and PDF page requests. Choose `server` for the
`llama-server` HTTP fallback or `executable` for the older per-request
`llama-uocr-parity` process.

## Run On Linux / WSL2

From the repository root:

```sh
uv run --project unlimited-ocr-portable baidu-uocr-client \
  --smoke --image dataset/sc-02.png --max-tokens 64

uv run --project unlimited-ocr-portable baidu-uocr-client \
  --host 127.0.0.1 --port 7861
```

Open `http://127.0.0.1:7861`.

The default paths are:

- `thirdparty/llama.cpp/build/bin/llama-server`
- `thirdparty/llama.cpp/build/bin/llama-uocr-parity`
- `thirdparty/llama.cpp/build/bin/libuocr-ffi.so` on Linux,
  `libuocr-ffi.dylib` on macOS, or `uocr-ffi.dll` on Windows
- `models/Unlimited-OCR-Q4_K_M.gguf`
- `models/mmproj-Unlimited-OCR-F16.gguf`

Override them when needed:

```sh
UOCR_LLAMA_SERVER_BIN=/path/to/llama-server \
UOCR_LLAMA_BIN=/path/to/llama-uocr-parity \
UOCR_FFI_LIB=/path/to/libuocr-ffi.so \
UOCR_MODEL=/path/to/Unlimited-OCR-Q4_K_M.gguf \
UOCR_MMPROJ=/path/to/mmproj-Unlimited-OCR-F16.gguf \
uv run --project unlimited-ocr-portable baidu-uocr-client
```

## Run On macOS

Build the patched llama.cpp branch with Metal, then launch through the macOS
portable scripts:

```sh
./scripts/mac/setup-build.sh

./scripts/mac/run-demo.sh --smoke --image dataset/sc-02.png --max-tokens 64

./scripts/mac/run-demo.sh --host 127.0.0.1 --port 7861
```

The macOS setup writes `uocr-runtime-env.sh`, which points the client at:

- `thirdparty/llama.cpp/build/bin/llama-uocr-parity`
- `models/Unlimited-OCR-Q4_K_M.gguf`
- `models/mmproj-Unlimited-OCR-F16.gguf`

## Run On Windows

Build the patched llama.cpp branch with CUDA, then set the native binary path:

```powershell
$env:UOCR_LLAMA_BIN = "thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe"
$env:UOCR_MODEL = "models\Unlimited-OCR-Q4_K_M.gguf"
$env:UOCR_MMPROJ = "models\mmproj-Unlimited-OCR-F16.gguf"

uv run --project unlimited-ocr-portable baidu-uocr-client `
  --host 127.0.0.1 --port 7861
```

The Windows path intentionally has no SGLang dependency. It still depends on
the patched native binary and the same GGUF files.

## Limitations

- Quality is still below the SGLang BF16 reference.
- Token streaming is best effort and depends on the native binary flushing
  generated stdout while decoding.
- Bounding boxes are parsed only from generated `<|det|>` / `<|ref|>` markers.
- Full-matrix validation was performed on WSL2; macOS should be smoke-tested on
  the target machine after building the Metal binary.
