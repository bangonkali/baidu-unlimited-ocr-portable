# Candidate-Best Client

This demo runs the current best portable Unlimited-OCR candidate through the
patched native llama.cpp binary. It does not use SGLang, PyTorch, Transformers,
or the Baidu custom SGLang wheel.

Default profile:

- `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full`
- Q4_K_M GGUF
- DeepSeek-OCR gundam preprocessing
- forced prompt EOS
- SGLang-style no-repeat ngram controls
- prefill-aware SWA128

The UI also exposes the exact-prefill/no-image-end/SWA128 profile as an
experimental option. That profile had higher average similarity in the full
matrix, but it produced empty rows and is not the default.

## Run On Linux / WSL2

From the repository root:

```sh
uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py \
  --smoke --image dataset/sc-02.png --max-tokens 64

uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py \
  --host 127.0.0.1 --port 7861
```

Open `http://127.0.0.1:7861`.

The default paths are:

- `thirdparty/llama.cpp/build/bin/llama-uocr-parity`
- `thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf`
- `thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf`

Override them when needed:

```sh
UOCR_LLAMA_BIN=/path/to/llama-uocr-parity \
UOCR_MODEL=/path/to/Unlimited-OCR-Q4_K_M.gguf \
UOCR_MMPROJ=/path/to/mmproj-Unlimited-OCR-F16.gguf \
uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py
```

## Run On Windows

Build the patched llama.cpp branch with CUDA, then set the native binary path:

```powershell
$env:UOCR_LLAMA_BIN = "thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe"
$env:UOCR_MODEL = "thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf"
$env:UOCR_MMPROJ = "thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf"

uv run --project unlimited-ocr-portable\candidate-best-client `
  unlimited-ocr-portable\candidate-best-client\app.py --host 127.0.0.1 --port 7861
```

The Windows path intentionally has no SGLang dependency. It still depends on
the patched native binary and the same GGUF files.

## Limitations

- Quality is still below the SGLang BF16 reference.
- Token streaming is best effort and depends on the native binary flushing
  generated stdout while decoding.
- Bounding boxes are parsed only from generated `<|det|>` / `<|ref|>` markers.
- WSL2 validation is the only runtime validation performed in this workspace.
