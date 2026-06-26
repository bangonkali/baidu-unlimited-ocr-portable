# Windows CUDA Candidate Validation

The SGLang BF16 reference is expected to run on WSL2/Linux. Windows validation
uses the same prepared dataset and compares native llama.cpp output against
reference JSON copied from WSL2.

The full executed validation procedure is documented in
`../TEST-PROCEDURE.md`. This Windows document focuses on reproducing the
candidate side natively and comparing it against the copied WSL2 reference.

Current validation status: the best patched Q4_K_M WSL2 run completed all 104
candidate rows with zero empty outputs, 56 automated passes, 17 repetition
rows, and average similarity 0.688. This is not production-ready yet, but it is
the current baseline Windows should reproduce before packaging work continues.

## Directory Layout

Use the same relative layout on Windows:

```text
unlimited-ocr/
  dataset/
  thirdparty/
    llama.cpp/
    uocr-gguf/
      Unlimited-OCR-Q4_K_M.gguf
      mmproj-Unlimited-OCR-F16.gguf
  unlimited-ocr-portable/
```

Copy these from WSL2 if needed:

- `unlimited-ocr-portable/results/manifest.jsonl`
- `unlimited-ocr-portable/results/prepared/`
- `unlimited-ocr-portable/results/reference/sglang/`
- `unlimited-ocr-portable/results/artifacts/reference/sglang-processor/`
- `unlimited-ocr-portable/results/artifacts/reference/sglang-native/`

These are the same artifacts named in `../TEST-PROCEDURE.md` under the Windows
candidate comparison flow.

## Build llama.cpp With CUDA

Install:

- Visual Studio 2022 Build Tools with C++ workload.
- CMake.
- NVIDIA CUDA toolkit compatible with the installed driver.
- Git.

From a Developer PowerShell:

```powershell
cmake -B thirdparty\llama.cpp\build `
  -S thirdparty\llama.cpp `
  -DGGML_CUDA=ON `
  -DCMAKE_BUILD_TYPE=Release

cmake --build thirdparty\llama.cpp\build --config Release `
  --target llama-mtmd-cli llama-uocr-parity llama-server
```

Stable unpatched llama-server is not enough for the current best candidate.
Build from the same custom llama.cpp branch validated on Linux:

```text
uocr-deepseek-ocr-parity
48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner
8fbbd5b mtmd-cli: add OCR sampling parity controls
3ebff83 mtmd: add Unlimited-OCR gundam grid parity
```

This branch is still a focused llama.cpp patch set, not a long-lived
llama-server product fork. If packaging proceeds before the patches are
upstreamed, prefer a small C++ wrapper over llama.cpp APIs or a pinned patched
llama.cpp build.

## Run Windows Candidate

Install `uv` for Windows, then from the repository root:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-mtmd-cli.exe `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf
```

If paths differ, pass explicit `--manifest` and `--results` paths.

Keep strategy outputs separate with `--candidate-engine`, especially when
testing Q5_K_M, Q6_K, BF16, repeat penalties, or image-token settings:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --candidate-engine llamacpp-q6_k `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-mtmd-cli.exe `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q6_K.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf `
  --quantization Q6_K `
  --repeat-penalty 1.05
```

Exact DeepSeek-OCR gundam smoke:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-gundam-exact-prefix-tight `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-mtmd-cli.exe `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf `
  --deepseek-ocr-mode gundam `
  --media-placement prefix-tight `
  --max-tokens 1024
```

Current best parity candidate:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --profiles grounding,plain_text,ocr_boxes,document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-windows `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-mtmd-cli.exe `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf `
  --max-tokens 8192 `
  --ctx-size 32768 `
  --deepseek-ocr-mode gundam `
  --deepseek-ocr-force-prompt-eos `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --force
```

Candidate-side artifact smoke:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-windows `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf `
  --deepseek-ocr-mode gundam `
  --deepseek-ocr-force-prompt-eos `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --debug-artifacts `
  --force
```

After copying `results\artifacts\reference\sglang` from WSL2, compare the
Windows candidate artifact with the WSL2 SGLang artifact:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare-artifacts `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-windows `
  --summary unlimited-ocr-portable\SUMMARY-parity-artifacts-windows.md
```

Exact-prefill diagnostic artifact:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-windows `
  --model thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf `
  --mmproj thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf `
  --deepseek-ocr-mode gundam `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --deepseek-ocr-no-image-end `
  --debug-artifacts `
  --max-tokens 1 `
  --force
```

After copying WSL2 `sglang-processor` and `sglang-native` artifacts, compare
runtime-token parity and one-token native logprob parity:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\SUMMARY-runtime-parity-windows.md

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --reference-engine sglang-native `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\SUMMARY-parity-artifacts-native-windows.md
```

The Linux run showed exact prefill and first-token top-k parity for this mode,
but not multi-token quality parity. Use it as a diagnostic, not as the current
best packaging setting.

## Compare Against WSL2 Reference

After copying `results/reference/sglang` from WSL2 and running the Windows
candidate:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare
```

The comparator will use the copied reference results and native Windows
candidate results to regenerate:

- `unlimited-ocr-portable\results\compare\metrics.csv`
- `unlimited-ocr-portable\SUMMARY.md`

Use the status definitions in `../TEST-PROCEDURE.md` when reviewing the Windows
comparison output.

The WSL2 reference and Windows candidate are intentionally decoupled. This lets
Windows validation run without a Python/SGLang stack while still comparing
against the BF16 oracle produced on Linux.

## Packaging Direction

- For experiments, call the pinned patched `llama-mtmd-cli` directly.
- For a native product, prefer a small C++ wrapper over llama.cpp APIs once
  output quality is acceptable. The wrapper should own image loading, prompt
  profile selection, output JSON, process exit codes, and platform packaging.
- Keep CUDA enabled on both Linux and Windows builds. CPU-only runs may be useful
  for debugging but are not the performance target.
- Full BF16 GGUF did not beat Q4 in the latest WSL2 run: 54 / 104 passes and
  average similarity 0.649 versus Q4's 56 / 104 and 0.688. Windows work should
  prioritize reproducing the exact patched Q4 behavior before broader
  packaging.
- The patched gundam path now combines local crop embeddings into SGLang's
  single local grid and passes the `sc-02` smoke. The five-case target set still
  has repetition failures, so Windows validation should compare against those
  same WSL2 reference outputs before treating the runtime as ready.
- `llama-uocr-parity` is a named native debug runner over the same MTMD CLI
  path. It should be used when comparing Windows candidate token traces against
  copied WSL2 SGLang chat-logprob artifacts.
- The current 104-row WSL2 Q4 run has no empty outputs but still fails on
  repetition, low-similarity, and bbox-count drift. Windows should reproduce
  this patched full-run behavior before packaging work continues.

## Notes

- Keep the same prepared PNG files on both platforms. This avoids differences
  from EXIF orientation, BMP/WEBP decoding, or PDF rendering.
- Copy `unlimited-ocr-portable\results\inspection\preprocessing.jsonl` with the
  reference artifacts if reviewing image-token parity on Windows.
- Windows validation should not attempt to run SGLang unless the SGLang stack is
  separately ported.
- If Q4_K_M quality is weak, repeat the same harness run with Q5_K_M, Q6_K, or
  BF16 GGUF by changing `--model`.
