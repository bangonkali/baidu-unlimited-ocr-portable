# Unlimited-OCR Portable Test Procedure

This document records the validation procedure executed on the local WSL2
Ubuntu environment for comparing:

- SGLang BF16 reference output from Baidu's custom wheel.
- llama.cpp GGUF candidate output from `Unlimited-OCR-Q4_K_M.gguf`.

The current result summary is in `SUMMARY.md`. The root engineering log is in
`../JOURNAL.md`.

## Scope

This procedure validates output quality and runtime behavior for a fixed local
dataset. It checks text, layout markers, repetition behavior, runtime latency,
and GPU memory snapshots.

It does not prove final production readiness for all quantizations or all
platforms. The current patched Q4_K_M run is materially better than the native
baseline, but still fails enough rows that packaging should stay gated on more
runtime/model parity work.

## Current Full Validation Result

The latest complete validation used all 26 prepared cases and four prompt
profiles: `grounding`, `plain_text`, `ocr_boxes`, and `document_parsing`.

Current custom llama.cpp branch:

```text
thirdparty/llama.cpp branch: uocr-deepseek-ocr-parity
7b0ec28 mtmd-cli: dump OCR output embedding summaries
48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner
8fbbd5b mtmd-cli: add OCR sampling parity controls
3ebff83 mtmd: add Unlimited-OCR gundam grid parity
9d5d882 model : Add label for LFM2.5-230M (#25008)
```

Current main candidate command:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full \
  --deepseek-ocr-mode gundam \
  --deepseek-ocr-force-prompt-eos \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full \
  --summary unlimited-ocr-portable/SUMMARY.md
```

Current main result in `SUMMARY.md`:

- Reference files: 104 / 104.
- Candidate files: 104 / 104.
- Candidate empty outputs: 0 / 104.
- Automated passes: 56 / 104.
- High-repetition rows: 17 / 104.
- Low-similarity rows: 14 / 104.
- Bbox-count mismatch rows: 14 / 104.
- Review rows: 3 / 104.
- Average text similarity: 0.688.
- Average candidate elapsed: 3809 ms.
- Average candidate GPU-after-request snapshot: 1528 MB.

BF16 GGUF was also rerun as a quality ceiling with the same full reference set:
`SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md`. It did not improve
the main decision: 54 / 104 passes, 27 repetition rows, average similarity
0.649, and average candidate elapsed 9743 ms.

## Tested Environment

Run from:

```sh
cd /home/ubuntu/projects/unlimited-ocr
```

Local environment used for the recorded run:

- OS: Ubuntu 24.04 under WSL2.
- GPU: NVIDIA GeForce RTX 5090.
- CUDA toolkit: 12.9.
- Dataset: `/home/ubuntu/projects/unlimited-ocr/dataset`.
- SGLang Python environment: `/home/ubuntu/projects/unlimited-ocr/.venv`.
- Custom SGLang wheel:
  `unlimited-ocr/wheel/sglang-0.0.0.dev11416+g92e8bb79e-py3-none-any.whl`.
- llama.cpp checkout: `thirdparty/llama.cpp`, tested branch
  `uocr-deepseek-ocr-parity`, current commit `7b0ec28`.
- llama.cpp binaries:
  `thirdparty/llama.cpp/build/bin/llama-mtmd-cli`,
  `thirdparty/llama.cpp/build/bin/llama-uocr-parity`, and
  `thirdparty/llama.cpp/build/bin/llama-server`.
- Candidate GGUF:
  `thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf`.
- Multimodal projector:
  `thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf`.

Verify the basic tools:

```sh
nvidia-smi
uv --version
thirdparty/llama.cpp/build/bin/llama-mtmd-cli --version
uv run --project unlimited-ocr-portable uocr-harness --help
```

## Step 1: Repair SGLang Reference Environment

The SGLang reference must use Baidu's bundled custom wheel. Generic SGLang or
newer public `sglang-kernel` builds caused ABI errors during this validation.

Realign `.venv` to the custom wheel stack:

```sh
uv pip install --python .venv/bin/python \
  ./unlimited-ocr/wheel/sglang-0.0.0.dev11416+g92e8bb79e-py3-none-any.whl \
  'torch==2.9.1' \
  'torchaudio==2.9.1' \
  'torchvision==0.24.1' \
  'sglang-kernel==0.4.1' \
  'transformers==5.3.0' \
  'kernels==0.11.7' \
  'pymupdf==1.27.2.2' \
  ninja
```

Verify ABI-sensitive imports:

```sh
uv run --no-project \
  --python .venv/bin/python \
  - <<'PY'
import torch
import sglang
import sgl_kernel
from sgl_kernel import sgl_per_token_quant_fp8
from sglang.srt.sampling.custom_logit_processor import DeepseekOCRNoRepeatNGramLogitProcessor

print("torch", torch.__version__, "cuda", torch.version.cuda, torch.cuda.is_available())
print("sglang", getattr(sglang, "__version__", "unknown"))
print("sgl_kernel", getattr(sgl_kernel, "__version__", "unknown"))
print("processor", DeepseekOCRNoRepeatNGramLogitProcessor.__name__)
print("fp8_func", sgl_per_token_quant_fp8.__name__)
PY
```

Expected verified versions from the recorded run:

```text
torch 2.9.1+cu128
sglang 0.0.0.dev11416+g92e8bb79e
sgl_kernel 0.4.1
```

## Step 2: Build llama.cpp With CUDA

Use the tested llama.cpp checkout unless the validation run intentionally tests a
newer commit.

```sh
uv tool run cmake -B thirdparty/llama.cpp/build \
  -S thirdparty/llama.cpp \
  -DGGML_CUDA=ON \
  -DCMAKE_BUILD_TYPE=Release

uv tool run cmake --build thirdparty/llama.cpp/build -j \
  --target llama-mtmd-cli llama-uocr-parity llama-server
```

If CMake is not on PATH but the build directory already exists, the generated
Makefile can rebuild the same targets:

```sh
make -C thirdparty/llama.cpp/build llama-mtmd-cli llama-uocr-parity llama-server -j8
```

Stable upstream llama-server can load the community Unlimited-OCR GGUF and
`mmproj`, but it cannot reproduce the current patched SGLang-style gundam path.
The current workspace uses a local custom llama.cpp branch, not a separate
llama-server product fork. The shared MTMD patch is used by both
`llama-mtmd-cli`, `llama-uocr-parity`, and `llama-server`; the no-repeat,
forced prompt EOS, prefill-aware SWA, min-new-token, and debug artifact controls
are currently wired through the MTMD CLI path.

## Step 3: Prepare Dataset

Normalize source images and render PDF pages:

```sh
uv run --project unlimited-ocr-portable uocr-harness prepare --force
```

Recorded output:

```text
Prepared 26 cases -> unlimited-ocr-portable/results/manifest.jsonl
```

The harness:

- Converts image inputs to prepared PNGs.
- Applies EXIF orientation.
- Renders PDF pages at 300 DPI.
- Preserves PDF page order.

## Step 4: SGLang Reference Smoke Test

Run a small reference check before the full pass:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server --limit 1 --max-tokens 128 --force
```

The harness starts SGLang with:

- `--trust-remote-code`
- `--enable-custom-logit-processor`
- `--attention-backend flashinfer`
- `--context-length 32768`
- `--page-size 1`
- `--disable-overlap-schedule`
- `--skip-server-warmup`

Why `flashinfer`: on the recorded RTX 5090 / SM120 system, FA3 startup failed
because this SGLang build restricts FA3 to SM80-SM90.

Why `.venv/bin` is on `PATH`: FlashInfer JIT needs `ninja`; the harness prepends
the selected SGLang interpreter's `bin` directory when launching the server.

Expected smoke output:

```text
Wrote 2 SGLang result files
```

Reference server logs are written to:

```text
unlimited-ocr-portable/results/logs/sglang_server.log
```

## Step 5: Full SGLang Reference Run

Run the BF16 reference over all prepared cases and all four validation profiles:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 --force
```

Recorded output:

```text
Wrote 104 SGLang result files
```

The default harness profiles are `grounding` and `plain_text`. The full
validation explicitly adds `ocr_boxes` and `document_parsing`.

Expected result location:

```text
unlimited-ocr-portable/results/reference/sglang/<case_id>/<profile>.json
```

Recorded reference outcome:

- Result files: 104 / 104.
- Request errors: 0.
- Empty normalized outputs: 0.
- Average elapsed in metrics: 5797 ms per row.
- Average GPU-after-request snapshot: 31654 MB.
- Server log ended with graceful shutdown.

## Step 6: Full Patched llama.cpp Q4_K_M Candidate Run

Run the best current Q4 candidate over the same manifest, prompts, and token
budget:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full \
  --deepseek-ocr-mode gundam \
  --deepseek-ocr-force-prompt-eos \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --force
```

The harness invokes `llama-mtmd-cli` with:

- `--chat-template deepseek-ocr`
- `--temp 0`
- `--top-k 1`
- `-ngl all`
- `-n 8192`
- `-c 32768`
- `--image <prepared_png>`
- `--mmproj thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf`
- `-m thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf`
- `LLAMA_DEEPSEEK_OCR_GUNDAM=1`
- `LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM=1`
- `LLAMA_DEEPSEEK_OCR_NGRAM_SIZE=30`
- `LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW=90`
- `LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST=128821,128822`
- `LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA=1`
- `LLAMA_DEEPSEEK_OCR_DECODE_WINDOW=128`
- `--override-kv tokenizer.ggml.add_eos_token=bool:true`

Recorded output:

```text
Wrote 104 llama.cpp result files
```

Expected result location:

```text
unlimited-ocr-portable/results/candidate/llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full/<case_id>/<profile>.json
```

Recorded candidate outcome:

- Result files: 104 / 104.
- Process errors: 0.
- Empty normalized outputs: 0 / 104.
- High-repetition rows: 17 / 104.
- Automated passes: 56 / 104.
- Low-similarity rows: 14 / 104.
- Bbox-count mismatch rows: 14 / 104.
- Average text similarity: 0.688.
- Average elapsed in metrics: 3809 ms per row.
- Average GPU-after-request snapshot: 1528 MB.

The candidate GPU snapshot underreports peak memory because `llama-mtmd-cli`
exits after each row. Active spot checks during candidate generation showed
roughly 9-12 GB transient VRAM use.

## Step 7: Compare and Summarize

Regenerate comparison metrics and Markdown summary:

```sh
uv run --project unlimited-ocr-portable uocr-harness compare
```

Recorded output:

```text
Wrote 104 comparison rows -> unlimited-ocr-portable/SUMMARY.md
```

Generated files:

```text
unlimited-ocr-portable/results/compare/metrics.csv
unlimited-ocr-portable/SUMMARY.md
```

Current status counts:

```text
bbox_count_mismatch: 14
candidate_repetition: 17
low_similarity: 14
pass: 56
review: 3
```

Interpretation:

- `candidate_empty` means the process exited successfully but normalized output
  text was empty.
- `candidate_repetition` means normalized output had a repeated 4-gram ratio of
  at least 0.35.
- `bbox_count_mismatch` means candidate bbox marker count differed from the
  reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.

## Step 8: Verify Counts

Optional checks after a full run:

```sh
find unlimited-ocr-portable/results/reference/sglang -type f -name '*.json' | wc -l
find unlimited-ocr-portable/results/candidate/llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full -type f -name '*.json' | wc -l
uv run --project unlimited-ocr-portable -m compileall unlimited-ocr-portable/uocr_harness
```

Expected counts for the recorded dataset:

```text
104
104
```

## Step 9: Parity Artifact Instrumentation Smoke

The custom branch includes a native MTMD debug runner target,
`llama-uocr-parity`. It uses the same implementation as `llama-mtmd-cli`, but
is built as a named checkpoint for parity instrumentation. When
`LLAMA_UOCR_PARITY_DUMP` is set, the native runner writes prompt/media token
counts, image-embedding summaries, prefill top logits, and per-token generation
top-k data.

Capture a SGLang chat-logprob artifact for the same reference path used by the
normal reference run:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 128 \
  --debug-artifacts \
  --debug-top-logprobs 8 \
  --force
```

Capture the matching patched llama.cpp artifact:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 128 \
  --timeout-s 1200 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-smoke \
  --deepseek-ocr-mode gundam \
  --deepseek-ocr-force-prompt-eos \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --debug-artifacts \
  --debug-top-k 8 \
  --force
```

Compare the artifacts:

```sh
uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-smoke \
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-smoke.md
```

Recorded artifact finding:

- SGLang's first API-visible token is `<|det|>`.
- llama.cpp's raw first token is newline token `201`; the next visible token is
  `<|det|>`.
- The baseline candidate's first-token top-k did not include `<|det|>` in the
  captured top 8.
- Repeating the smoke with `--deepseek-ocr-no-image-end` still produced raw
  newline first, but moved `<|det|>` into rank 6 of the top-k list. The artifact
  summary is `SUMMARY-parity-artifacts-noimgend-smoke.md`.
- The prior five-case target run showed `--deepseek-ocr-no-image-end` was worse
  overall, so it remains a diagnostic switch rather than a default.

These artifacts are sufficient for API-visible token comparison. They do not
expose SGLang internal image embedding tensors or token IDs through the public
chat endpoint; deeper SGLang server instrumentation would be required for exact
hidden-state/logit parity.

## Windows Candidate Comparison Flow

SGLang BF16 reference generation is expected to run on WSL2/Linux. For Windows
candidate validation:

1. Copy these WSL2 outputs to the same relative paths on Windows:
   - `unlimited-ocr-portable/results/manifest.jsonl`
   - `unlimited-ocr-portable/results/prepared/`
   - `unlimited-ocr-portable/results/reference/sglang/`
2. Build llama.cpp with CUDA on Windows.
3. Run `uocr-harness run-llamacpp` with Windows paths to
   `llama-mtmd-cli.exe`, GGUF, and `mmproj`.
4. Run `uocr-harness compare`.

This keeps the BF16 reference fixed while allowing native candidate runs on
other platforms.

## Current Decision From This Procedure

Q4_K_M is not production-ready with the current patched llama.cpp invocation.
The earlier empty-output failure mode is fixed in the best Q4 full run, but the
remaining failures still show repetition, low similarity, and bbox marker drift.

This does not prove llama.cpp is the wrong portable runtime. It means the next
validation pass should focus below the harness layer: model/runtime numeric
parity, deeper SGLang decoding behavior, and upstreamable llama.cpp support for
the model-specific DeepSeek-OCR/Unlimited-OCR path.

Stable unpatched llama-server is not enough for this model path today. The
practical packaging base is the local custom llama.cpp branch. A small C++
wrapper over llama.cpp APIs remains the preferred product packaging direction
after output quality is acceptable, unless the focused patches are upstreamed
first.

## Follow-up Strategy Matrix

After the full Q4_K_M run, these targeted strategy checks were executed on five
representative problem cases:

- `613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4`
- `chinese-paper-page-0001-2200885e`
- `chinese-paper-page-0002-3d10e38a`
- `sc-02-45a8efac`
- `upside-left-9e645a2a`

Strategy summaries:

| Strategy | Summary | Decision |
|---|---|---|
| Q4_K_M repeat penalty 1.05 | `SUMMARY-q4-rp105.md` | Did not fix empty/repetition failures. |
| Q5_K_M | `SUMMARY-q5_k_m.md` | Higher quant still failed on the representative target set. |
| Q6_K | `SUMMARY-q6_k.md` | Higher quant still failed on the representative target set. |
| BF16 GGUF | `SUMMARY-bf16.md` | Full precision still failed, so this is not just quantization loss. |
| Prompt matrix | `SUMMARY-q4-prompts.md` | Removed empty rows but caused severe repetition. |
| llama-server Q4 | `SUMMARY-llamacpp-server-q4.md` | Native `/completion` works mechanically but quality still fails. |
| Image token smoke | `SUMMARY-image-tokens-smoke.md` | Harness can pass image-token bounds; tuning remains unresolved. |
| DeepSeek-OCR gundam smoke | `SUMMARY-deepseekocr-gundam-smoke.md` | Opt-in crop mode executes and improves one-row prompt-only similarity, but still repeats. |
| DeepSeek-OCR gundam + repeat penalty | `SUMMARY-deepseekocr-gundam-rp105-smoke.md` | Similarity improved further on one row, but repetition remains over threshold. |
| Exact DeepSeek-OCR gundam smoke | `SUMMARY-deepseekocr-gundam-exact-smoke.md` | SGLang local-grid composition passes the `sc-02` smoke. |
| Exact DeepSeek-OCR gundam + repeat penalty smoke | `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md` | Also passes the `sc-02` smoke, but no better than no repeat penalty. |
| Exact DeepSeek-OCR gundam target set | `SUMMARY-deepseekocr-gundam-exact-target-doc.md` | Best target-set result so far: 3 pass, 2 repetition. |
| Exact DeepSeek-OCR gundam target set + repeat penalty | `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md` | Worse than no repeat penalty: 2 pass, 3 repetition. |
| Exact DeepSeek-OCR gundam full run | `SUMMARY-deepseekocr-gundam-exact-full.md` | Historical exact-grid-only run: improved baseline to 8 pass, but still had 36 empty rows. |
| Q4 forced-EOS + SGLang no-repeat + SWA128 full run | `SUMMARY.md` | Best zero-empty full run: 56 pass, 0 empty, 17 repetition, average similarity 0.688. |
| Output embedding artifact smoke | `SUMMARY-parity-artifacts-output-embeddings-onetok.md` | SGLang hidden-state summary and llama.cpp output-embedding summaries are both present on the one-token exact-prefill smoke. |
| BF16 forced-EOS + SGLang no-repeat full run | `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md` | Quality ceiling check did not beat Q4: 54 pass, 27 repetition, average similarity 0.649. |
| Parity artifact smoke | `SUMMARY-parity-artifacts-smoke.md` | SGLang visible first token is `<|det|>`; llama.cpp raw first token is newline then `<|det|>`. |
| Parity artifact no-image-end smoke | `SUMMARY-parity-artifacts-noimgend-smoke.md` | No-image-end moves `<|det|>` into top-k but still emits newline first and was worse on the target set. |

The strategy details and commands are recorded in `JOURNAL.md` under
`2026-06-26 Follow-up Strategy Matrix`.

## DeepSeek-OCR / SGLang Gundam Parity Procedure

This procedure validates the focused llama.cpp MTMD patch for SGLang gundam
parity.

Inspect all prepared cases:

```sh
uv run --project unlimited-ocr-portable uocr-harness inspect-preprocessing
```

Recorded output:

```text
Wrote 26 preprocessing inspection rows -> /home/ubuntu/projects/unlimited-ocr/unlimited-ocr-portable/results/inspection/preprocessing.jsonl
```

Important finding from that file:

- Native llama.cpp DeepSeek-OCR preprocessing uses one 1024x1024 padded global
  image, which is 273 image tokens for this model.
- SGLang gundam uses a 1024 global image plus 640 local crop tiles.
- All 26 current prepared cases use a multi-column SGLang crop grid.
- The patched llama.cpp gundam path creates matching 640 tiles and one 1024
  overview, then composes local tile embeddings into one combined local grid.
  The inspector reports `llamacpp_gundam_layout_exact: true`.
- Independent-tile counts remain in the inspection output as diagnostics. For
  example, the browser screenshot would have had 3353 independent-tile image
  tokens, but the composed SGLang/llama.cpp total is 3113.

Run the exact focused smoke:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 1024 \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-prefix-tight \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-prefix-tight \
  --summary unlimited-ocr-portable/SUMMARY-deepseekocr-gundam-exact-smoke.md
```

Run the exact repeat-penalty variant:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 1024 \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-rp105-prefix-tight \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --repeat-penalty 1.05 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-rp105-prefix-tight \
  --summary unlimited-ocr-portable/SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md
```

Run the exact five-case target set:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles document_parsing \
  --max-tokens 8192 \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-target-doc \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-target-doc \
  --summary unlimited-ocr-portable/SUMMARY-deepseekocr-gundam-exact-target-doc.md
```

Run the same target set with repeat penalty:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles document_parsing \
  --max-tokens 8192 \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-rp105-target-doc \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --repeat-penalty 1.05 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-rp105-target-doc \
  --summary unlimited-ocr-portable/SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md
```

Run the exact full 52-row comparison over the baseline prompt profiles:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-full \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-full \
  --summary unlimited-ocr-portable/SUMMARY-deepseekocr-gundam-exact-full.md
```

Recorded results:

| Candidate | Similarity | Repetition | Status |
|---|---:|---:|---|
| `llamacpp-q4_k_m-prompts` | 0.011 | 0.773 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-prefix-tight` | 0.237 | 0.530 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-rp105-prefix-tight` | 0.458 | 0.588 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-exact-prefix-tight` | 0.998 | 0.191 | `pass` |
| `llamacpp-q4_k_m-gundam-exact-rp105-prefix-tight` | 0.997 | 0.190 | `pass` |

Target-set summaries:

| Candidate | Rows | Status counts | Average similarity | Notes |
|---|---:|---|---:|---|
| `llamacpp-q4_k_m-gundam-exact-target-doc` | 5 | 3 pass, 2 repetition | 0.568 | Best target-set result so far. Failures were the browser screenshot and rotated image. |
| `llamacpp-q4_k_m-gundam-exact-rp105-target-doc` | 5 | 2 pass, 3 repetition | 0.428 | Worse than no repeat penalty; one paper page overproduced bbox markers. |
| `llamacpp-q4_k_m-gundam-exact-full` | 52 | 8 pass, 36 empty, 5 repetition, 1 bbox mismatch, 1 malformed, 1 review | 0.691 | Full baseline-profile comparison. Improves over native Q4 baseline but remains not production-ready. |

Conclusion:

- The local-grid blocker was feasible and is now implemented in C/C++ inside
  llama.cpp's DeepSeek-OCR MTMD path.
- The exact path is a major improvement over prompt-only and independent-tile
  gundam modes.
- It is still not production quality on the target set. Remaining failures are
  likely outside local-grid composition: SGLang custom no-repeat/logit
  processing, decoding behavior, prompt/profile handling, orientation-sensitive
  cases, or another model-specific runtime detail.

## Latest Parity-Control Experiments

After exact local-grid composition, the remaining target-set work isolated and
implemented these additional controls on the local custom branch:

- SGLang custom no-repeat processor parity in `llama-mtmd-cli`:
  ngram size 30, window 90, whitelist tokens `128821,128822`.
- Origin-token tracking for the no-repeat processor, including text prefill
  tokens and a media sentinel for image-token spans.
- Forced prompt EOS harness switch:
  `--deepseek-ocr-force-prompt-eos`.
- Prefill-aware decode-window experiment:
  `--deepseek-ocr-prefill-aware-swa --deepseek-ocr-decode-window 128`.
- Image-end newline suppression experiment:
  `--deepseek-ocr-no-image-end`.
- EOG min-new-token diagnostic:
  `--deepseek-ocr-min-new-tokens N`.

SGLang source findings:

- The custom wheel uses
  `DeepseekOCRNoRepeatNGramLogitProcessor` with defaults 30 / 90 and whitelist
  `[128821, 128822]`.
- The Unlimited-OCR HF processor strips the prompt EOS before inference, even
  though it constructs the sequence with `eos=True` internally.
- The SGLang conversation template is an empty-role, empty-separator template
  with image token at prefix, so the effective prompt is `<image>` immediately
  followed by the prompt text.
- The GGUF files do not carry the SGLang `sliding_window_size=128` metadata,
  so prefill-aware SWA remains an explicit experiment in the candidate.

Target-set results on five cases and four profiles:

| Candidate | Rows | Status counts | Average similarity | Notes |
|---|---:|---|---:|---|
| `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-target` | 20 | 8 pass, 8 repetition, 3 low similarity, 1 bbox mismatch | 0.464 | Forced EOS removed empties but did not solve repetition. |
| `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-target` | 20 | 9 pass, 6 repetition, 4 low similarity, 1 bbox mismatch | 0.502 | Best Q4 target setting. |
| `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-noimgend-target` | 20 | 8 pass, 10 repetition, 1 low similarity, 1 malformed-marker row | 0.476 | Suppressing the DeepSeek-OCR image-end newline was worse. |
| `llamacpp-q4_k_m-uocr-parity-origin-ngram-default-minnew1-target` | 20 | 7 pass, 10 repetition, 2 low similarity, 1 bbox mismatch | 0.417 | Banning immediate EOG removed empties but increased repetition. |
| `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-target` | 20 | 10 pass, 7 repetition, 3 low similarity | 0.513 | BF16 helped the target set slightly. |
| `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-swa128-target` | 20 | 10 pass, 7 repetition, 3 low similarity | 0.508 | SWA did not improve BF16 target quality. |

Full 104-row results:

| Candidate | Rows | Status counts | Average similarity | Average candidate ms | Decision |
|---|---:|---|---:|---:|---|
| `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full` | 104 | 56 pass, 17 repetition, 14 low similarity, 14 bbox mismatch, 3 review | 0.688 | 3809 | Current best candidate, still not production-ready. |
| `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-full` | 104 | 54 pass, 27 repetition, 9 low similarity, 6 bbox mismatch, 6 review, 2 malformed-marker rows | 0.649 | 9743 | BF16 did not beat Q4 full-run quality. |
| `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-full` | 104 | 49 pass, 27 repetition, 10 bbox mismatch, 7 review, 6 low similarity, 5 empty | 0.671 | 4719 | Exact prefill/no-image-end regressed on the full matrix; keep it diagnostic. |
| `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full` | 104 | 56 pass, 17 low similarity, 14 bbox mismatch, 7 repetition, 5 review, 5 empty | 0.717 | 3075 | Ties pass count and improves average similarity, but introduces empty rows. |

Packaging implication:

- Stable unpatched llama-server cannot reproduce the current candidate behavior.
- The practical base is the local `uocr-deepseek-ocr-parity` branch until the
  MTMD and sampling controls are upstreamed or replaced by a small wrapper.
- `llama-server` currently benefits from the shared MTMD grid patch, but the
  no-repeat/SWA/forced-EOS controls were validated through `llama-mtmd-cli`.
  A production server path needs those semantics wired into server generation or
  moved into a small custom C++ wrapper over llama.cpp APIs.

## Runtime Token / Logprob Parity Procedure

This pass adds deeper runtime parity artifacts after the native
`llama-uocr-parity` runner. It separates three questions:

- Does SGLang's processor/template input sequence match llama.cpp's prefill?
- Does SGLang native `/generate` first-token logprob/top-k match llama.cpp?
- Does exact prefill parity improve multi-token OCR quality?

Generate the SGLang processor/template artifact with the repo `.venv` custom
wheel:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m uocr_harness.cli inspect-sglang-processor \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --image-mode gundam \
  --media-placement separate \
  --force
```

Compare that processor artifact against existing and new llama.cpp native
artifacts:

```sh
uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-smoke \
  --summary unlimited-ocr-portable/SUMMARY-runtime-parity-smoke.md

uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noeos-smoke \
  --summary unlimited-ocr-portable/SUMMARY-runtime-parity-noeos-smoke.md

uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-smoke \
  --summary unlimited-ocr-portable/SUMMARY-runtime-parity-noimgend-smoke.md
```

Run the exact-prefill smoke candidate:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-smoke \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end \
  --debug-artifacts \
  --max-tokens 1024 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-smoke \
  --summary unlimited-ocr-portable/SUMMARY-runtime-parity-noimgend-noeos-smoke.md
```

Recorded runtime-token results for `sc-02-45a8efac` / `document_parsing`:

| Candidate | Candidate text tokens | Prefill delta | Status |
|---|---|---:|---|
| `llamacpp-q4_k_m-uocr-parity-debug-smoke` | `[0, 201, 34030, 76466, 16, 1]` | 2 | `candidate_extra_boundary_tokens` |
| `llamacpp-q4_k_m-uocr-parity-debug-noeos-smoke` | `[0, 201, 34030, 76466, 16]` | 1 | `candidate_extra_boundary_tokens` |
| `llamacpp-q4_k_m-uocr-parity-debug-noimgend-smoke` | `[0, 34030, 76466, 16, 1]` | 1 | `candidate_extra_boundary_tokens` |
| `llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-smoke` | `[0, 34030, 76466, 16]` | 0 | `runtime_sequence_match` |

The SGLang processor input length is 1517: 1513 image tokens plus the four
non-image tokens `[0, 34030, 76466, 16]`.

Capture a native SGLang `/generate` artifact with input logprobs. This starts
and stops the BF16 reference server:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m uocr_harness.cli run-sglang \
  --start-server \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 1 \
  --debug-native-artifacts \
  --debug-top-logprobs 5 \
  --force
```

Capture the matching one-token llama.cpp artifact and compare:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-onetok \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end \
  --debug-artifacts \
  --max-tokens 1 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-onetok \
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-native-onetok.md
```

Recorded native-logprob result:

- SGLang native `/generate` prompt tokens: 1517.
- SGLang captured `input_token_logprobs`: 1517 rows.
- SGLang first output token: `<|det|>` / token `128818`.
- llama.cpp exact-prefill first output token: `<|det|>` / token `128818`.
- First-output top-k overlap: 1.000.

Validate hidden-state return plumbing in an isolated results directory so the
normal SGLang reference output is not overwritten:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m uocr_harness.cli run-sglang \
  --start-server \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 1 \
  --debug-native-artifacts \
  --debug-return-hidden-states \
  --enable-return-hidden-states \
  --debug-top-logprobs 3 \
  --results /tmp/uocr-hidden-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --force
```

Recorded hidden-state result:

- Native artifact status: HTTP 200, no harness error.
- Prompt tokens: 1517.
- Completion tokens: 1.
- Summarized hidden-state shape: `[1, 1517, 1280]`.

Capture a matching llama.cpp one-token artifact with output-embedding summaries
enabled. This uses the patched `llama-uocr-parity` binary at commit `7b0ec28`
and writes into the same isolated results directory:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-output-embeddings-onetok \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-no-image-end \
  --debug-artifacts \
  --debug-output-embeddings \
  --debug-top-k 8 \
  --max-tokens 1 \
  --results /tmp/uocr-hidden-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --results /tmp/uocr-hidden-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-output-embeddings-onetok \
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-output-embeddings-onetok.md
```

Recorded output-embedding result:

- Native llama.cpp version: `5 (7b0ec28)`.
- Candidate prefill: 1517 tokens, matching the SGLang prompt token count.
- Candidate output embedding summaries: 2 rows.
- Candidate prefill-last output embedding width: 1280.
- Candidate generated-token output embedding count: 1.
- First-token visible/logit parity still holds: `<|det|>` and top-k overlap
  1.000.

Run the exact-prefill target set:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target \
  --summary unlimited-ocr-portable/SUMMARY-uocr-parity-q4-noimgend-noeos-target.md
```

Recorded target result:

| Candidate | Rows | Status counts | Average similarity | Notes |
|---|---:|---|---:|---|
| `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target` | 20 | 10 pass, 4 repetition, 6 low similarity | 0.512 | Exact prefill and first-token top-k parity slightly improve the Q4 target set, but output remains not production-ready. |

Run 64-token generation-step artifacts in an isolated results directory. This
keeps the full SGLang reference output under `results/reference/sglang` intact:

```sh
rm -rf /tmp/uocr-step-results

PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m uocr_harness.cli run-sglang \
  --start-server \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 64 \
  --debug-native-artifacts \
  --debug-top-logprobs 8 \
  --results /tmp/uocr-step-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --force

uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-64tok \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end \
  --debug-artifacts \
  --debug-top-k 8 \
  --max-tokens 64 \
  --results /tmp/uocr-step-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-generation-artifacts \
  --results /tmp/uocr-step-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-64tok \
  --summary unlimited-ocr-portable/SUMMARY-generation-steps-noimgend-noeos-64tok.md
```

The same procedure was repeated for:

- `llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-noswa-64tok`
- `llamacpp-q5_k_m-uocr-parity-debug-noimgend-noeos-64tok`
- `llamacpp-q6_k-uocr-parity-debug-noimgend-noeos-64tok`
- `llamacpp-bf16-uocr-parity-debug-noimgend-noeos-64tok`

Recorded generation-step result for `sc-02-45a8efac` / `document_parsing`:

| Candidate | Matching prefix | First divergence | First divergent tokens | Average top-k overlap | Decision |
|---|---:|---:|---|---:|---|
| Q4 exact prefill | 3 | 3 | SGLang `91` vs Q4 `92` | 0.807 | Later-token rank flip after `<|det|>header [`; keep as diagnostic. |
| Q4 exact prefill, no SWA experiment | 3 | 3 | SGLang `91` vs Q4 `92` | 0.807 | SWA flag is not the cause of the first divergence. |
| Q5_K_M exact prefill | 1 | 1 | SGLang `header` vs Q5 `aside` | 0.059 | Higher quantization regresses the step trace. |
| Q6_K exact prefill | 1 | 1 | SGLang `header` vs Q6 `aside` | 0.059 | Higher quantization regresses the step trace. |
| BF16 GGUF exact prefill | 1 | 1 | SGLang `header` vs BF16 `aside` | 0.061 | BF16 GGUF does not close runtime parity. |

Run the exact-prefill/no-image-end full Q4 matrix only after the target-set
smoke, because it is slower and the no-SWA variant regressed:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-full \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-no-image-end

uv run --project unlimited-ocr-portable uocr-harness compare \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --reference-engine sglang \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-full \
  --summary unlimited-ocr-portable/SUMMARY-uocr-parity-q4-noimgend-noeos-full.md
```

Recorded full exact-prefill result:

- 104 candidate rows and 104 reference rows were present.
- Status counts: 49 pass, 27 repetition, 10 bbox mismatch, 7 review, 6 low
  similarity, 5 empty.
- Average similarity: 0.671.
- Decision: do not promote this no-SWA exact-prefill variant.

Run the exact-prefill/no-image-end/SWA128 full matrix as a separate candidate:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end

uv run --project unlimited-ocr-portable uocr-harness compare \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --reference-engine sglang \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full \
  --summary unlimited-ocr-portable/SUMMARY-uocr-parity-q4-noimgend-noeos-swa128-full.md
```

Recorded full exact-prefill/SWA128 result:

- 104 candidate rows and 104 reference rows were present.
- Status counts: 56 pass, 17 low similarity, 14 bbox mismatch, 7 repetition,
  5 review, 5 empty.
- Average similarity: 0.717.
- Average candidate latency: 3075 ms.
- Decision: useful alternate candidate because it reduces repetition and
  improves average similarity, but not production parity because it still has
  empty outputs and low-similarity rows.

Conclusion:

- Tokenizer/template/media-token parity is now implemented and validated for the
  smoke case.
- First-token SGLang native `/generate` logits/top-k align with llama.cpp under
  exact prefill.
- Generation-step parity diverges immediately after the first stable OCR prefix
  on Q4 and even earlier for Q5_K_M, Q6_K, and BF16 GGUF.
- Exact prefill/no-image-end without SWA regresses on the full 104-row matrix.
- Exact prefill/no-image-end/SWA128 ties the current 56-pass count and improves
  average similarity, but it still has 5 empty rows.
- Multi-token output parity is still not achieved. The remaining likely gap is
  later-token runtime drift: attention/KV behavior, hidden-state divergence,
  numerical differences, or model-runtime implementation differences after the
  first generated token.

## Candidate-Best Client Demo Procedure

This procedure validates the interactive demo under
`unlimited-ocr-portable/candidate-best-client`. The demo does not use SGLang; it
invokes the patched native `llama-uocr-parity` binary and streams generated
stdout into a Gradio UI.

Best default profile:

- `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full`
- Reason: 56 / 104 pass, 0 empty rows, average similarity 0.688.

Experimental profile exposed in the UI:

- `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full`
- Reason: average similarity 0.717, but 5 empty rows, so not the default.

Compile the demo package:

```sh
uv run --project unlimited-ocr-portable/candidate-best-client \
  -m compileall \
  unlimited-ocr-portable/candidate-best-client/app.py \
  unlimited-ocr-portable/candidate-best-client/uocr_candidate_client
```

Run a short native smoke for the default profile:

```sh
uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py \
  --smoke --image dataset/sc-02.png --max-tokens 64
```

Recorded WSL2 result on 2026-06-27:

- Exit code: 0.
- Native elapsed: 2050 ms.
- Output began with visible OCR markers:
  `<|det|>header [92, 24, 139, 55]<|/det|>Sci`.

Run a short smoke for the experimental profile:

```sh
uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py \
  --smoke --image dataset/sc-02.png \
  --profile experimental-exact-prefill-q4 \
  --max-tokens 64
```

Recorded WSL2 result:

- Exit code: 0.
- Native elapsed: 2438 ms.
- Output began with visible OCR markers:
  `<|det|>header [92, 24, 139, 53]<|/det|>Sci`.

Validate PDF rendering and overlay parsing:

```sh
PYTHONPATH=unlimited-ocr-portable/candidate-best-client \
uv run --project unlimited-ocr-portable/candidate-best-client python -c \
"from pathlib import Path; from uocr_candidate_client.pdf import pdf_to_images; from uocr_candidate_client.parsing import build_preview_image, extract_boxes; pages = pdf_to_images(Path('dataset/chinese-paper.pdf'), dpi=72); text = '<|det|>header [10, 20, 100, 120]<|/det|>'; preview = build_preview_image(pages[0], text); print({'pages': len(pages), 'boxes': len(extract_boxes(text)), 'preview': preview is not None})"
```

Recorded result:

```text
{'pages': 6, 'boxes': 1, 'preview': True}
```

Launch the UI:

```sh
uv run --project unlimited-ocr-portable/candidate-best-client \
  unlimited-ocr-portable/candidate-best-client/app.py \
  --host 127.0.0.1 --port 7861
```

Verify the endpoint:

```sh
curl -sSf http://127.0.0.1:7861/ | rg -n "Unlimited-OCR Portable Candidate|gradio"
```

Recorded result:

- Gradio served `http://127.0.0.1:7861`.
- The returned config contained the title `Unlimited-OCR Portable Candidate`.
- The default slider value was 8192 tokens.
- The queued `run_ocr` endpoint was present.

Windows note:

- The same client can run on Windows after building the patched native
  `llama-uocr-parity.exe`; set `UOCR_LLAMA_BIN`, `UOCR_MODEL`, and
  `UOCR_MMPROJ` as documented in `docs/WINDOWS.md`.
