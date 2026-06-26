# Windows CUDA Quick Start

This guide is for running the portable Unlimited-OCR candidate natively on
Windows without SGLang. It uses:

- `bangonkali/llama.cpp-baidu-unlimited-ocr`, branch
  `uocr-deepseek-ocr-parity`.
- `bangonkali/baidu-unlimited-ocr-portable`, branch `main`.
- GGUF model files from `sahilchachra/Unlimited-OCR-GGUF`.
- CUDA-enabled `llama-uocr-parity.exe`.

The current default candidate is:

```text
llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full
```

It is the best user-facing demo default because the WSL2 full run produced
56 / 104 passes, 0 empty outputs, 17 repetition rows, and average similarity
0.688. It is not production parity with SGLang.

## 1. Install Prerequisites

Install these on Windows:

- Git for Windows.
- Visual Studio 2022 Build Tools with the C++ workload.
- CMake.
- NVIDIA driver and CUDA Toolkit.
- `uv`.
- Hugging Face CLI as `hf`.

Recommended PowerShell install commands for the tools that are available through
`winget`:

```powershell
winget install --id Git.Git -e
winget install --id Kitware.CMake -e
winget install --id Astral-sh.UV -e
```

Install Visual Studio Build Tools with the C++ workload from the Visual Studio
installer. Install the CUDA Toolkit from NVIDIA and make sure `nvcc` is on PATH.

Install or update the Hugging Face CLI through `uv` if `hf` is not already
available:

```powershell
uv tool install "huggingface-hub[cli]"
```

Verify from a fresh Developer PowerShell for VS 2022:

```powershell
git --version
cmake --version
uv --version
hf version
nvcc --version
nvidia-smi
```

Authenticate to Hugging Face if needed. This guide assumes the user is already
authorized:

```powershell
hf auth whoami

# Only if not already authenticated:
hf auth login
```

## 2. Create The Workspace

Use this layout. The portable app defaults assume `thirdparty` and
`unlimited-ocr-portable` are siblings.

```text
C:\uocr\
  dataset\
  thirdparty\
    llama.cpp\
    uocr-gguf\
  unlimited-ocr-portable\
```

Clone the repos:

```powershell
mkdir C:\uocr
cd C:\uocr
mkdir thirdparty

git clone -b uocr-deepseek-ocr-parity `
  git@github.com:bangonkali/llama.cpp-baidu-unlimited-ocr.git `
  thirdparty\llama.cpp

git clone git@github.com:bangonkali/baidu-unlimited-ocr-portable.git `
  unlimited-ocr-portable
```

Do not put GGUF files in either Git repo. Keep them under
`thirdparty\uocr-gguf`.

## 3. Download Required GGUF Assets

Every run needs two files:

- One language model GGUF.
- The shared F16 vision projector: `mmproj-Unlimited-OCR-F16.gguf`.

Download the required default Q4_K_M model and projector:

```powershell
mkdir thirdparty\uocr-gguf

hf download sahilchachra/Unlimited-OCR-GGUF `
  Unlimited-OCR-Q4_K_M.gguf `
  mmproj-Unlimited-OCR-F16.gguf `
  --local-dir thirdparty\uocr-gguf
```

Optional quality/diagnostic downloads:

```powershell
hf download sahilchachra/Unlimited-OCR-GGUF `
  Unlimited-OCR-Q5_K_M.gguf `
  Unlimited-OCR-Q6_K.gguf `
  Unlimited-OCR-BF16.gguf `
  --local-dir thirdparty\uocr-gguf
```

Optional extra Sahil quants, not yet validated by this project:

```powershell
hf download sahilchachra/Unlimited-OCR-GGUF `
  Unlimited-OCR-Q8_0.gguf `
  Unlimited-OCR-Q5_K_S.gguf `
  Unlimited-OCR-Q4_K_S.gguf `
  Unlimited-OCR-Q3_K_M.gguf `
  Unlimited-OCR-IQ4_XS.gguf `
  Unlimited-OCR-IQ4_NL.gguf `
  Unlimited-OCR-IQ3_M.gguf `
  Unlimited-OCR-IQ3_XXS.gguf `
  Unlimited-OCR-IQ2_M.gguf `
  Unlimited-OCR.imatrix `
  --local-dir thirdparty\uocr-gguf
```

Verify the required files:

```powershell
Get-Item thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf
Get-Item thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf
```

Expected approximate sizes:

- `Unlimited-OCR-Q4_K_M.gguf`: 1.9 GB.
- `mmproj-Unlimited-OCR-F16.gguf`: 775 MB.

## 4. Build Patched llama.cpp With CUDA

Run from a Developer PowerShell for VS 2022:

```powershell
cd C:\uocr

cmake -B thirdparty\llama.cpp\build `
  -S thirdparty\llama.cpp `
  -G "Visual Studio 17 2022" `
  -A x64 `
  -DGGML_CUDA=ON `
  -DCMAKE_BUILD_TYPE=Release

cmake --build thirdparty\llama.cpp\build `
  --config Release `
  --target llama-mtmd-cli llama-uocr-parity llama-server `
  -j
```

Confirm the binaries:

```powershell
Get-ChildItem thirdparty\llama.cpp\build -Recurse -Filter llama-uocr-parity.exe
Get-ChildItem thirdparty\llama.cpp\build -Recurse -Filter llama-mtmd-cli.exe
Get-ChildItem thirdparty\llama.cpp\build -Recurse -Filter llama-server.exe
```

The expected Visual Studio path is usually:

```text
thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe
```

This custom branch includes the base DeepSeek-OCR support plus project-specific
patches:

```text
uocr-deepseek-ocr-parity
7b0ec28 mtmd-cli: dump OCR output embedding summaries
48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner
8fbbd5b mtmd-cli: add OCR sampling parity controls
3ebff83 mtmd: add Unlimited-OCR gundam grid parity
```

Stock upstream llama.cpp after PR #17400 can load DeepSeek-OCR-family GGUFs,
but it does not include the validated Unlimited-OCR gundam grid/no-repeat/SWA
debug behavior from this custom branch.

## 5. Set Runtime Paths

Set these in the same PowerShell session before running the demo or harness:

```powershell
$env:UOCR_LLAMA_BIN = "C:\uocr\thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe"
$env:UOCR_MODEL = "C:\uocr\thirdparty\uocr-gguf\Unlimited-OCR-Q4_K_M.gguf"
$env:UOCR_MMPROJ = "C:\uocr\thirdparty\uocr-gguf\mmproj-Unlimited-OCR-F16.gguf"
```

If your build emits binaries somewhere else, point `UOCR_LLAMA_BIN` at the path
found by `Get-ChildItem`.

## 6. Prepare A Test Image

The Git repos do not include the private/local dataset. For a quick Windows
smoke, either:

- Copy `dataset\sc-02.png` from the WSL2 workspace into `C:\uocr\dataset`, or
- Put any test document image at `C:\uocr\dataset\document.png`.

Example:

```powershell
mkdir C:\uocr\dataset
```

The smoke commands below assume:

```text
C:\uocr\dataset\sc-02.png
```

Change the image path if you use a different file.

## 7. Run A Native CLI Smoke

This directly runs the patched native binary with the current default candidate
settings.

```powershell
cd C:\uocr

$env:LLAMA_DEEPSEEK_OCR_GUNDAM = "1"
$env:LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM = "1"
$env:LLAMA_DEEPSEEK_OCR_NGRAM_SIZE = "30"
$env:LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW = "90"
$env:LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST = "128821,128822"
$env:LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA = "1"
$env:LLAMA_DEEPSEEK_OCR_DECODE_WINDOW = "128"
Remove-Item Env:\LLAMA_DEEPSEEK_OCR_NO_IMAGE_END -ErrorAction SilentlyContinue

& $env:UOCR_LLAMA_BIN `
  -m $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --image dataset\sc-02.png `
  -p "<__media__>document parsing." `
  --chat-template deepseek-ocr `
  --temp 0 `
  --top-k 1 `
  -n 64 `
  -c 32768 `
  -ngl all `
  --log-verbosity 2 `
  --override-kv tokenizer.ggml.add_eos_token=bool:true
```

Expected result: non-empty OCR output, usually with visible `<|det|>` markers.

## 8. Run The Candidate-Best Client Demo

The Gradio demo lives in:

```text
unlimited-ocr-portable\candidate-best-client
```

Run a short smoke through the Python wrapper:

```powershell
cd C:\uocr

uv run --project unlimited-ocr-portable\candidate-best-client `
  unlimited-ocr-portable\candidate-best-client\app.py `
  --smoke --image dataset\sc-02.png --max-tokens 64
```

Launch the UI:

```powershell
uv run --project unlimited-ocr-portable\candidate-best-client `
  unlimited-ocr-portable\candidate-best-client\app.py `
  --host 127.0.0.1 --port 7861
```

Open:

```text
http://127.0.0.1:7861
```

The UI supports:

- Image upload.
- PDF upload and page selection.
- Prompt profile selection.
- Default zero-empty Q4 profile.
- Experimental exact-prefill/no-image-end/SWA128 profile.
- Streaming OCR text from the native subprocess.
- Parsed bounding-box preview when `<|det|>` / `<|ref|>` markers are present.

## 9. Run The Portable Harness Candidate

Use this when you want persisted JSON outputs under
`unlimited-ocr-portable\results`.

The harness reads images from `C:\uocr\dataset` and writes normalized inputs to
`unlimited-ocr-portable\results\prepared`. Run `prepare` once after copying or
changing the dataset:

```powershell
cd C:\uocr

uv run --project unlimited-ocr-portable uocr-harness prepare
```

Small smoke:

```powershell
cd C:\uocr

uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --profiles document_parsing `
  --max-tokens 64 `
  --ctx-size 32768 `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-windows-smoke `
  --deepseek-ocr-mode gundam `
  --deepseek-ocr-force-prompt-eos `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --force
```

Current best full candidate profile:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --profiles grounding,plain_text,ocr_boxes,document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-windows `
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

If testing other downloaded models, change `UOCR_MODEL` and
`--candidate-engine`. Example:

```powershell
$env:UOCR_MODEL = "C:\uocr\thirdparty\uocr-gguf\Unlimited-OCR-Q6_K.gguf"
```

## 10. Troubleshooting

If CMake cannot find CUDA:

- Confirm `nvcc --version` works in the same Developer PowerShell.
- Confirm the NVIDIA driver is installed with `nvidia-smi`.
- Re-run CMake after fixing PATH.

If the native binary cannot load the model:

- Confirm you built the `uocr-deepseek-ocr-parity` branch.
- Confirm `--chat-template deepseek-ocr` and `--mmproj` are present.
- Confirm the GGUF and mmproj paths point to real files.

If `hf download` fails:

- Run `hf auth whoami`.
- Run `hf auth login` if not authenticated.
- Check that the model repo is reachable:
  `hf download sahilchachra/Unlimited-OCR-GGUF README.md --local-dir thirdparty\uocr-gguf`.

If PowerShell blocks scripts:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

If the client launches but OCR is blank:

- First run the native CLI smoke in section 7.
- Reduce `--max-tokens` to 64 for quick diagnosis.
- Check that `UOCR_LLAMA_BIN`, `UOCR_MODEL`, and `UOCR_MMPROJ` are set in the
  same terminal session.

## Validation And Reference Notes

The sections below are not required for the first Windows run. They document the
comparison workflow used to compare Windows candidate output against WSL2
SGLang reference artifacts.

## Reference Artifact Layout

The SGLang BF16 reference is expected to run on WSL2/Linux. Windows validation
uses the same prepared dataset and compares native llama.cpp output against
reference JSON copied from WSL2.

Copy these from WSL2 if needed:

- `unlimited-ocr-portable/results/manifest.jsonl`
- `unlimited-ocr-portable/results/prepared/`
- `unlimited-ocr-portable/results/reference/sglang/`
- `unlimited-ocr-portable/results/artifacts/reference/sglang-processor/`
- `unlimited-ocr-portable/results/artifacts/reference/sglang-native/`
- Optional generation-step summaries such as
  `SUMMARY-generation-steps-noimgend-noeos-64tok.md` for native token-trace
  comparison.

The full executed validation procedure is documented in
`../TEST-PROCEDURE.md`.

## Candidate-Side Artifact Smoke

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-windows `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --deepseek-ocr-mode gundam `
  --deepseek-ocr-force-prompt-eos `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --debug-artifacts `
  --force
```

After copying WSL2 reference artifacts, compare:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare-artifacts `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-windows `
  --summary unlimited-ocr-portable\SUMMARY-parity-artifacts-windows.md
```

## Generation-Step Comparison

For native `/generate` step traces copied from WSL2:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare-generation-artifacts `
  --results unlimited-ocr-portable\results `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --reference-engine sglang-native `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\SUMMARY-generation-steps-windows.md
```

## Exact-Prefill Diagnostic Artifact

This is diagnostic only, not the default packaging path:

```powershell
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-windows `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --deepseek-ocr-mode gundam `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --deepseek-ocr-no-image-end `
  --debug-artifacts `
  --debug-output-embeddings `
  --max-tokens 1 `
  --force
```

After copying WSL2 `sglang-processor` and `sglang-native` artifacts:

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
but not multi-token quality parity.

## Compare Against WSL2 Reference

After copying `results/reference/sglang` from WSL2 and running the Windows
candidate:

```powershell
uv run --project unlimited-ocr-portable uocr-harness compare
```

The comparator writes:

- `unlimited-ocr-portable\results\compare\metrics.csv`
- `unlimited-ocr-portable\SUMMARY.md`

Use the status definitions in `../TEST-PROCEDURE.md` when reviewing the Windows
comparison output.

## Known Validation Status

- The current full WSL2 Q4 run has no empty outputs but still fails on
  repetition, low-similarity, and bbox-count drift.
- Full BF16 GGUF did not beat Q4 in the latest WSL2 run: 54 / 104 passes and
  average similarity 0.649 versus Q4's 56 / 104 and 0.688.
- Exact-prefill/no-image-end Q4 is not the default. It regressed on the full
  WSL2 matrix to 49 / 104 passes, 5 empty rows, 27 repetition rows, and average
  similarity 0.671.
- Exact-prefill/no-image-end/SWA128 tied the 56 / 104 pass count and improved
  average similarity to 0.717, but still had 5 empty rows and 17 low-similarity
  rows.
- The patched gundam path combines local crop embeddings into SGLang's single
  local grid and passes the `sc-02` smoke. Larger target sets still have
  repetition failures.
- WSL2 generation-step comparison shows Q4 exact-prefill matches SGLang through
  `<|det|>header [` and then diverges at the first bbox coordinate (`91` vs
  `92`). Q5_K_M, Q6_K, and BF16 diverge earlier at `header` vs `aside`, so
  Windows validation should not assume a higher GGUF fixes parity.
- WSL2 output-embedding smoke captured SGLang hidden shape `[1, 1517, 1280]`
  and llama.cpp prefill/generation output embeddings with width 1280.

## Packaging Direction

- For experiments, call the pinned patched native binary directly.
- For a native product, prefer a small C++ wrapper over llama.cpp APIs once
  output quality is acceptable.
- Keep CUDA enabled on both Linux and Windows builds. CPU-only runs are useful
  for debugging but are not the performance target.
- Windows validation should not attempt to run SGLang unless the SGLang stack is
  separately ported.
