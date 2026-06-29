# Windows CUDA Quick Start

This guide covers the Windows native path. The C++/React workbench is the
target product path; the Python Gradio app remains a reference/demo for
behavior and runtime parity checks.

## Release Zip Path

The preferred Windows onboarding path is the GitHub Release zip:

```text
uocr-workbench-windows-x64-<tag>.zip
```

Extract the zip anywhere and run `uocr-server.exe`. The server binds to
`127.0.0.1:8765`, hosts the React workbench from `web/`, and opens the browser
automatically. The folder is self-contained except for downloaded GGUF model
files under `models\`.

Logs are appended to `logs\uocr-server.log` inside the extracted folder and are
also printed to the terminal running `uocr-server.exe`. The React workbench can
download the default Unlimited-OCR Q4_K_M GGUF pair into `models\`; images and
rendered PDF pages are sent through the bundled CUDA `uocr-ffi.dll` once those
files are present. Multi-page PDFs are rendered in-process by MuPDF embedded in
`uocr-server.exe`; the portable zip does not ship `mutool.exe`.

Optional authenticated Hugging Face download:

```powershell
$env:HF_TOKEN = "hf_..."
.\uocr-server.exe
```

`HF_TOKEN` is read only by the server process. If it is absent, the server falls
back to `HUGGING_FACE_HUB_TOKEN`, then to public downloads. Token values are not
displayed, persisted, or logged.

First run sequence:

1. Open **Models** and click **Download missing**.
2. Click **Choose Folder** to use the trusted native Windows folder picker, or
   paste a path into the fallback text box.
3. Click **Start Scan**.
4. Watch the progress in the toolbar, the Diagnostics pane, the terminal, or
   `logs\uocr-server.log`.

The Models panel shows the required GGUF files, auth status, current file,
bytes downloaded versus total bytes, percentage, MiB/s, ETA, retry,
re-download, and cancel. Cancelling keeps the `.download` partial file so a
later retry can resume when Hugging Face supports range requests.

The browser keeps one websocket connection to `ws://127.0.0.1:8765/api/events`.
Model progress, scan progress, new OCR text, bounding boxes, document status,
and log entries update through that single channel. Folder selection, model
download commands, and scan commands still use normal HTTP API requests.

One-line install to `~\.uocr`:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/bangonkali/baidu-unlimited-ocr-portable/main/scripts/install.ps1 | iex"
```

Uninstall by deleting `~\.uocr`.

Check the release tag associated with an executable:

```powershell
~\.uocr\uocr-server.exe --version
```

## C++ Workbench Path

Build the dependency-light C++ core and tests first:

```powershell
cmake -S . -B build\uocr-server -DUOCR_BUILD_SERVER=OFF -DUOCR_BUILD_TESTS=ON
cmake --build build\uocr-server --config Release --target uocr-core-tests
ctest --test-dir build\uocr-server -C Release --output-on-failure
```

Build the React SPA:

```powershell
cd src\uocr-client
bun install
bun run build
bun run build-storybook
cd ..\..
```

The supported local workbench build script prepares the React app, builds the
MuPDF static libraries from `thirdparty\mupdf`, links them into
`uocr-server.exe`, and copies the React build to `web\`:

```powershell
.\scripts\windows\build-workbench.ps1
```

When Drogon and Trantor are available through the Windows CMake toolchain, the
lower-level CMake path is:

```powershell
cmake -S . -B build\uocr-server-drogon -DUOCR_BUILD_SERVER=ON
cmake --build build\uocr-server-drogon --config Release --target uocr-server
```

That lower-level path assumes the MuPDF static libraries already exist under
`thirdparty\mupdf\platform\win32\x64\Release\`. The server binds to
`127.0.0.1` by default and serves `/api/*`, `/api/openapi.json`, and the built
React app in the release layout.

Create the same zip layout that GitHub Actions publishes:

```powershell
.\scripts\windows\package-workbench.ps1 -Version v0.0.9
```

The output is written to `dist\uocr-workbench-windows-x64-v0.0.9.zip` plus a
`.sha256` file.

## Python Reference Path

The reference/demo path runs the portable Unlimited-OCR candidate natively on
Windows without SGLang. It uses:

- `bangonkali/llama.cpp-baidu-unlimited-ocr`, branch
  `uocr-deepseek-ocr-parity`.
- `bangonkali/baidu-unlimited-ocr-portable`, branch `main`.
- GGUF model files from `sahilchachra/Unlimited-OCR-GGUF`.
- CUDA-enabled `llama-uocr-parity.exe`, downloaded from GitHub Releases by
  default or built locally on request.

The current default candidate is:

```text
llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full
```

It is the practical user-facing Q4 demo default because the current R-SWA WSL2
full run produced 54 / 104 passes, 0 empty outputs, 19 repetition rows, and
average similarity 0.678. BF16 with R-SWA is the current best pass-count result
at 61 / 104 passes, but it is slower, heavier, and still not production parity
with SGLang.

## Scripted Quick Start

Start from PowerShell with an NVIDIA driver visible through `nvidia-smi` and
CUDA runtime libraries compatible with CUDA 13 binaries. The prebuilt runtime
label is `windows-x86_64-cuda13`. The CUDA 13 runtime supports compute
capability 7.5 or newer and includes Blackwell RTX 5090 support (`sm_120`,
built as `120a-real`).

For local source builds, use **Visual Studio 2026 Developer PowerShell
v18.8.0-insiders**, CMake 4.2.1 or newer, and CUDA. The expected Windows CUDA
target for the next validation pass is CUDA 13.x; the current validated local
environment used CUDA 13.3, where `nvcc --version` includes:

```text
cuda_13.3.r13.3/compiler.37862127_0
```

Create the workspace, clone the portable repo recursively, and run the doctor
preflight first. Doctor does not download models or build native code; it
checks the repo, submodule, required command-line tools, Visual Studio shell,
CUDA/GPU visibility, Hugging Face auth when downloads are needed, lockfile, and
local model/build cache status.

```powershell
mkdir C:\uocr
cd C:\uocr

git clone --recursive git@github.com:bangonkali/baidu-unlimited-ocr-portable.git `
  unlimited-ocr-portable

cd C:\uocr\unlimited-ocr-portable

.\scripts\windows\setup-build.ps1 -Doctor
```

`-Doctor` is the canonical PowerShell spelling. The script also accepts
`--doctor` through a compatibility alias in hosts that bind double-dash
parameters.

When doctor has no blocking failures, run the full setup. It syncs Python
dependencies, downloads the default Q4_K_M GGUF model plus mmproj into
`models\`, installs the prebuilt `windows-x86_64-cuda13` runtime from GitHub
Releases, and writes runtime environment variables:

```powershell
.\scripts\windows\setup-build.ps1
```

To compile the CUDA runtime locally instead of downloading it:

```powershell
.\scripts\windows\setup-build.ps1 -RuntimeSource build
```

Local CUDA source builds use the same default architecture set as release
builds:

```text
75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real
```

Override it only when intentionally narrowing the binary for a specific machine:

```powershell
.\scripts\windows\setup-build.ps1 `
  -RuntimeSource build `
  -CudaArchitectures "120a-real"
```

To try download first and compile only if no release asset is available:

```powershell
.\scripts\windows\setup-build.ps1 -RuntimeSource auto
```

To also download the diagnostic Q5_K_M, Q6_K, and BF16 GGUFs:

```powershell
.\scripts\windows\setup-build.ps1 -IncludeDiagnostics
```

The setup script checks:

- `git`
- `uv`
- `hf`
- `nvidia-smi`
- Python/Gradio dependencies via `uv sync --frozen`.
- Hugging Face authorization via `hf auth whoami` when model downloads are needed.
- GGUF downloads into `models\`.
- downloaded or built `uocr-ffi.dll`, `llama-uocr-parity.exe`,
  `llama-mtmd-cli.exe`, and `llama-server.exe`
- required GGUF files under
  `C:\uocr\unlimited-ocr-portable\models`

When `-RuntimeSource build` is used, it also checks `cmake`, `cl.exe`, `nvcc`,
Visual Studio Developer PowerShell variables, and the `llama.cpp` submodule.

Useful setup switches:

- `-IncludeDiagnostics`: also download Q5_K_M, Q6_K, and BF16 GGUFs.
- `-ForceModelDownload`: redownload model files even when non-empty local
  files already exist.
- `-RuntimeSource download|build|auto`: choose prebuilt runtime download, local
  compilation, or download-with-build-fallback. Default: `download`.
- `-RuntimeVersion TAG`: download a specific GitHub Release tag instead of the
  latest release.
- `-ForceRuntimeDownload`: redownload and reinstall the prebuilt runtime.
- `-SkipPythonSync`: skip `uv sync --frozen` if you already synced the project.
- `-SkipModelDownload`: skip Hugging Face auth and model download.
- `-CudaArchitectures VALUE`: pass `CMAKE_CUDA_ARCHITECTURES` for local CUDA
  source builds. Default includes RTX 5090 / `sm_120`.
- `-SkipBuild`: skip CMake configure/build when using `-RuntimeSource build` or
  an `auto` fallback build.

The script writes:

```text
C:\uocr\unlimited-ocr-portable\uocr-runtime-env.ps1
```

Run a smoke test after copying a test image into
`C:\uocr\unlimited-ocr-portable\dataset`:

```powershell
.\scripts\windows\run-demo.ps1 `
  -Smoke `
  -Image C:\uocr\unlimited-ocr-portable\dataset\sc-02.png `
  -MaxTokens 64
```

Launch the Gradio demo:

```powershell
.\scripts\windows\run-demo.ps1 `
  -HostName 127.0.0.1 `
  -Port 7861
```

Open:

```text
http://127.0.0.1:7861
```

The UI defaults to the persistent `ffi` runtime backend. It loads
`uocr-ffi.dll` through `ctypes`, keeps the model and mmproj resident, and
processes all PDF pages through that native session. Use the runtime selector in
the header, or `baidu-uocr-client --smoke --runtime-backend executable`, to
force the legacy per-request executable path.

If PowerShell blocks local scripts:

```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

The rest of this document is the manual setup path and validation notes.

## 1. Install Prerequisites

Install these on Windows:

- Git for Windows.
- Visual Studio 2026 Developer PowerShell v18.8.0-insiders, or compatible
  Visual Studio C++ build tools.
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

Verify from a fresh Visual Studio 2026 Developer PowerShell:

```powershell
git --version
cmake --version
uv --version
hf --version
cl.exe
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

Use this layout. The portable app defaults keep git-based source dependencies
under `unlimited-ocr-portable\thirdparty` and downloaded model assets under
`unlimited-ocr-portable\models`.

```text
C:\uocr\
  unlimited-ocr-portable\
    dataset\
    models\            # downloaded HF assets, ignored by git
    thirdparty\
      llama.cpp\        # git submodule
```

Clone the portable repo recursively:

```powershell
mkdir C:\uocr
cd C:\uocr

git clone --recursive git@github.com:bangonkali/baidu-unlimited-ocr-portable.git `
  unlimited-ocr-portable

cd C:\uocr\unlimited-ocr-portable
```

If the repo was cloned without `--recursive`, initialize submodules:

```powershell
git submodule update --init --recursive
```

Do not commit GGUF files. Keep them under `models\`, which is ignored by git.

## 3. Download Required GGUF Assets

Every run needs two files:

- One language model GGUF.
- The shared F16 vision projector: `mmproj-Unlimited-OCR-F16.gguf`.

The scripted setup handles this automatically. Manual downloads should use the
same local model directory as the script:

```powershell
mkdir models

hf download sahilchachra/Unlimited-OCR-GGUF `
  Unlimited-OCR-Q4_K_M.gguf `
  --local-dir models

hf download sahilchachra/Unlimited-OCR-GGUF `
  mmproj-Unlimited-OCR-F16.gguf `
  --local-dir models
```

Optional quality/diagnostic downloads:

```powershell
foreach ($file in @(
  "Unlimited-OCR-Q5_K_M.gguf",
  "Unlimited-OCR-Q6_K.gguf",
  "Unlimited-OCR-BF16.gguf"
)) {
  hf download sahilchachra/Unlimited-OCR-GGUF `
    $file `
    --local-dir models
}
```

Optional extra Sahil quants, not yet validated by this project:

```powershell
foreach ($file in @(
  "Unlimited-OCR-Q8_0.gguf",
  "Unlimited-OCR-Q5_K_S.gguf",
  "Unlimited-OCR-Q4_K_S.gguf",
  "Unlimited-OCR-Q3_K_M.gguf",
  "Unlimited-OCR-IQ4_XS.gguf",
  "Unlimited-OCR-IQ4_NL.gguf",
  "Unlimited-OCR-IQ3_M.gguf",
  "Unlimited-OCR-IQ3_XXS.gguf",
  "Unlimited-OCR-IQ2_M.gguf",
  "Unlimited-OCR.imatrix"
)) {
  hf download sahilchachra/Unlimited-OCR-GGUF `
    $file `
    --local-dir models
}
```

Verify the required files:

```powershell
Get-Item models\Unlimited-OCR-Q4_K_M.gguf
Get-Item models\mmproj-Unlimited-OCR-F16.gguf
```

Expected approximate sizes:

- `Unlimited-OCR-Q4_K_M.gguf`: 1.9 GB.
- `mmproj-Unlimited-OCR-F16.gguf`: 775 MB.

## 4. Build Patched llama.cpp With CUDA

Run from Visual Studio 2026 Developer PowerShell:

```powershell
cd C:\uocr\unlimited-ocr-portable

cmake -B thirdparty\llama.cpp\build `
  -S thirdparty\llama.cpp `
  -DGGML_CUDA=ON `
  "-DCMAKE_CUDA_ARCHITECTURES=75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real" `
  -DCMAKE_BUILD_TYPE=Release

cmake --build thirdparty\llama.cpp\build `
  --config Release `
  --target llama-mtmd-cli llama-uocr-parity llama-server uocr-ffi `
  --parallel
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
f3e5dcccf deepseek2-ocr: add Unlimited-OCR R-SWA parity
7b0ec28 mtmd-cli: dump OCR output embedding summaries
48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner
8fbbd5b mtmd-cli: add OCR sampling parity controls
3ebff83 mtmd: add Unlimited-OCR gundam grid parity
```

Stock upstream llama.cpp after PR #17400 can load DeepSeek-OCR-family GGUFs,
but it does not include the validated Unlimited-OCR gundam grid/no-repeat/SWA
debug behavior from this custom branch. The branch also factors in PR #24975
style reference-SWA masking for DeepSeek-OCR/Unlimited-OCR.

## 5. Set Runtime Paths

Set these in the same PowerShell session before running the demo or harness:

```powershell
$env:UOCR_LLAMA_BIN = "C:\uocr\unlimited-ocr-portable\thirdparty\llama.cpp\build\bin\Release\llama-uocr-parity.exe"
$env:UOCR_MODEL = "C:\uocr\unlimited-ocr-portable\models\Unlimited-OCR-Q4_K_M.gguf"
$env:UOCR_MMPROJ = "C:\uocr\unlimited-ocr-portable\models\mmproj-Unlimited-OCR-F16.gguf"
```

If your build emits binaries somewhere else, point `UOCR_LLAMA_BIN` at the path
found by `Get-ChildItem`.

## 6. Prepare A Test Image

The Git repos do not include the private/local dataset. For a quick Windows
smoke, either:

- Copy `dataset\sc-02.png` from the WSL2 workspace into
  `C:\uocr\unlimited-ocr-portable\dataset`, or
- Put any test document image at
  `C:\uocr\unlimited-ocr-portable\dataset\document.png`.

Example:

```powershell
mkdir C:\uocr\unlimited-ocr-portable\dataset
```

The smoke commands below assume:

```text
C:\uocr\unlimited-ocr-portable\dataset\sc-02.png
```

Change the image path if you use a different file.

## 7. Run A Native CLI Smoke

This directly runs the patched native binary with the current default candidate
settings.

```powershell
cd C:\uocr\unlimited-ocr-portable

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

## 8. Run The Portable Client Demo

The Gradio demo lives in:

```text
unlimited-ocr-portable\src\baidu_unlimited_ocr_portable
```

Run a short smoke through the Python wrapper:

```powershell
cd C:\uocr\unlimited-ocr-portable

uv run --project . baidu-uocr-client `
  --smoke --image dataset\sc-02.png --max-tokens 64
```

Launch the UI:

```powershell
uv run --project . baidu-uocr-client `
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
- Practical zero-empty Q4 R-SWA profile.
- Experimental exact-prefill/no-image-end R-SWA profile.
- Streaming OCR text from the native subprocess.
- Parsed bounding-box preview when `<|det|>` / `<|ref|>` markers are present.

## 9. Run The Portable Harness Candidate

Use this when you want persisted JSON outputs under
`results`.

The harness reads images from `C:\uocr\unlimited-ocr-portable\dataset` and
writes normalized inputs to `results\prepared`. Run `prepare` once after
copying or changing the dataset:

```powershell
cd C:\uocr\unlimited-ocr-portable

uv run --project . uocr-harness prepare
```

Small smoke:

```powershell
cd C:\uocr\unlimited-ocr-portable

uv run --project . uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --profiles document_parsing `
  --max-tokens 64 `
  --ctx-size 32768 `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-windows-smoke `
  --deepseek-ocr-mode gundam `
  --deepseek-ocr-force-prompt-eos `
  --media-placement prefix-tight `
  --deepseek-ocr-no-repeat-ngram `
  --deepseek-ocr-prefill-aware-swa `
  --deepseek-ocr-decode-window 128 `
  --force
```

Current practical Q4 full candidate profile:

```powershell
uv run --project . uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --model $env:UOCR_MODEL `
  --mmproj $env:UOCR_MMPROJ `
  --profiles grounding,plain_text,ocr_boxes,document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-windows `
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
$env:UOCR_MODEL = "C:\uocr\unlimited-ocr-portable\models\Unlimited-OCR-Q6_K.gguf"
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
  `hf download sahilchachra/Unlimited-OCR-GGUF README.md --local-dir models`.

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
- Optional generation-step summaries under `analysis\summaries\`, such as
  `SUMMARY-generation-steps-noimgend-noeos-64tok.md` for native token-trace
  comparison.

The full executed validation procedure is documented in
`../TEST-PROCEDURE.md`.

## Candidate-Side Artifact Smoke

```powershell
uv run --project . uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-windows `
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
uv run --project . uocr-harness compare-artifacts `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-windows `
  --summary unlimited-ocr-portable\analysis\summaries\SUMMARY-parity-artifacts-windows.md
```

## Generation-Step Comparison

For native `/generate` step traces copied from WSL2:

```powershell
uv run --project . uocr-harness compare-generation-artifacts `
  --results unlimited-ocr-portable\results `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --reference-engine sglang-native `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\analysis\summaries\SUMMARY-generation-steps-windows.md
```

## Exact-Prefill Diagnostic Artifact

This is diagnostic only, not the default packaging path:

```powershell
uv run --project . uocr-harness run-llamacpp `
  --binary $env:UOCR_LLAMA_BIN `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-noimgend-noeos-windows `
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
uv run --project . uocr-harness compare-runtime-parity `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\analysis\summaries\SUMMARY-runtime-parity-windows.md

uv run --project . uocr-harness compare-artifacts `
  --case-id sc-02-45a8efac `
  --profiles document_parsing `
  --reference-engine sglang-native `
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-debug-noimgend-noeos-windows `
  --summary unlimited-ocr-portable\analysis\summaries\SUMMARY-parity-artifacts-native-windows.md
```

The Linux run showed exact prefill and first-token top-k parity for this mode,
but not multi-token quality parity.

## Compare Against WSL2 Reference

After copying `results/reference/sglang` from WSL2 and running the Windows
candidate:

```powershell
uv run --project . uocr-harness compare
```

The comparator writes:

- `unlimited-ocr-portable\results\compare\metrics.csv`
- `unlimited-ocr-portable\analysis\summaries\SUMMARY.md`

Use the status definitions in `../TEST-PROCEDURE.md` when reviewing the Windows
comparison output.

## Known Validation Status

- The current R-SWA full WSL2 Q4 run has no empty outputs but still fails on
  repetition, low-similarity, and bbox-count drift: 54 / 104 passes, 19
  repetition rows, and average similarity 0.678.
- The best current pass-count result is BF16 with R-SWA: 61 / 104 passes, 18
  repetition rows, average similarity 0.684, and average candidate latency
  6963 ms. It is not the default because it is slower, heavier, and still far
  from parity.
- Q5_K_M with R-SWA reached 59 / 104 passes, but had 21 repetition rows and
  average similarity 0.672.
- Q6_K with R-SWA reached 51 / 104 passes and is not currently useful as a
  quality improvement.
- Exact-prefill/no-image-end Q4 with R-SWA is not the default. It tied
  56 / 104 passes and improved average similarity to 0.719, but still had
  5 empty rows.
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
