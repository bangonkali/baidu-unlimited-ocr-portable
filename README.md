# Unlimited-OCR Portable

This directory contains local portable-runtime experiments and the validation
harness for comparing:

- SGLang BF16 reference output from WSL2/Linux.
- llama.cpp/GGUF candidate output from Linux, macOS, or Windows.

Generated harness outputs live under `results/` and are ignored by git.
Historical and current summaries live under `analysis/summaries/`. Start with
`analysis/summaries/SUMMARY-uocr-rswa-executive.md` for the latest R-SWA
decision, then use `analysis/summaries/SUMMARY.md` as the summary index.

Use `TEST-PROCEDURE.md` as the canonical runbook for the validation pass that
produced the current summary.

The uv project name is `baidu-unlimited-ocr-portable`.

## Repository Layout

- `src/baidu_unlimited_ocr_portable/`: Gradio/native-runtime demo client.
- `analysis/uocr_harness/`: validation, comparison, and artifact harness.
- `analysis/summaries/`: current and historical Markdown result summaries.
- `docs/`: Linux, macOS, and Windows setup notes.
- `scripts/mac/`: macOS setup/build and demo launch scripts.
- `scripts/windows/`: Windows setup/build and demo launch scripts.
- `thirdparty/llama.cpp/`: git submodule for the patched llama.cpp fork.
- `models/`: local HF model asset directory and cache, ignored by git.

## Clone Layout

For a fresh machine, clone recursively so git-based dependencies are present
under this repo:

```sh
git clone --recursive git@github.com:bangonkali/baidu-unlimited-ocr-portable.git
```

If the clone already exists:

```sh
git submodule update --init --recursive
```

The macOS and Windows setup scripts also run the submodule update, sync the
Python environment with `uv sync --frozen`, and download GGUF model files with
`hf` into `models/`.

Run the doctor first on macOS to verify prerequisites without downloading or
building:

```sh
./scripts/mac/setup-build.sh --doctor
```

Run the doctor first on Windows to verify prerequisites without downloading or
building:

```powershell
.\scripts\windows\setup-build.ps1 -Doctor
```

The script also accepts `--doctor` through a compatibility alias when supported
by the active PowerShell host.

See `docs/MACOS.md` and `docs/WINDOWS.md` for the platform-specific quick
starts.

## Validation Harness

Run from the repository root:

```sh
uv run --project unlimited-ocr-portable uocr-harness prepare

uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --limit 1 --max-tokens 256

uv run --project unlimited-ocr-portable uocr-harness compare
```

Inspect image-token preprocessing parity:

```sh
uv run --project unlimited-ocr-portable uocr-harness inspect-preprocessing
```

Run the SGLang reference against an already running server:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --limit 1 --max-tokens 256
```

Or let the harness start and stop SGLang:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server --limit 1 --max-tokens 256
```

For a full current validation run, remove `--limit 1`, use
`--max-tokens 8192`, and pass:
`--profiles grounding,plain_text,ocr_boxes,document_parsing`.

The latest full WSL2 run is summarized in
`analysis/summaries/SUMMARY-uocr-rswa-executive.md`. At the time of writing,
SGLang BF16 completed all 104 reference rows. The current Q4_K_M R-SWA
candidate completed all 104 rows with zero empty outputs, 54 automated passes,
19 repetition rows, and average similarity 0.678. BF16 R-SWA reached the best
pass count so far at 61 / 104, but remains slower and still not production
parity. Practical Q4 did not improve versus the older CLI-prune baseline.

For the exact commands, environment repair steps, expected file counts, and
interpretation rules, see `TEST-PROCEDURE.md`.

Follow-up targeted strategy summaries are also under `analysis/summaries/`:

- `SUMMARY-generation-steps-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok.md`
- `SUMMARY-generation-steps-q5-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q6-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-bf16-noimgend-noeos-64tok.md`
- `SUMMARY-runtime-parity-noimgend-noeos-smoke.md`
- `SUMMARY-parity-artifacts-native-onetok.md`
- `SUMMARY-parity-artifacts-output-embeddings-onetok.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-swa128-full.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-target.md`
- `SUMMARY-parity-artifacts-smoke.md`
- `SUMMARY-parity-artifacts-noimgend-smoke.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`
- `SUMMARY-bf16.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-full.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-executive.md`
- `SUMMARY-uocr-rswa-q4-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-q4-noimgend-noeos-full.md`
- `SUMMARY-uocr-rswa-q5_k_m-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-q6_k-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-bf16-eos-origin-ngram-default-full.md`

The strategy decisions are recorded in `JOURNAL.md`.

## DeepSeek-OCR Parity Controls

The harness now exposes prompt/media placement and an opt-in patched llama.cpp
DeepSeek-OCR gundam path. In this workspace the patched C++ MTMD path composes
local crop embeddings into the same SGLang-style local grid before appending the
1024 overview:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-prefix-tight \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --max-tokens 1024
```

This exact path passes the `sc-02` / `document_parsing` smoke with similarity
0.998 and matching bbox marker counts. Additional parity controls now include
SGLang-style no-repeat defaults, forced prompt EOS, core reference-SWA behavior,
and diagnostic image-end/min-new-token switches. The older CLI KV-pruning SWA
experiment is disabled unless `LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE=1` is
explicitly set.

The custom branch also builds `llama-uocr-parity`, a named MTMD CLI target for
debug instrumentation. It accepts the same harness invocation as
`llama-mtmd-cli`; with `--debug-artifacts`, the harness sets
`LLAMA_UOCR_PARITY_DUMP` and captures prompt/media token counts, prefill top
logits, and generation top-k data for comparison with SGLang chat logprobs:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --binary thirdparty/llama.cpp/build/bin/llama-uocr-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-smoke \
  --deepseek-ocr-mode gundam \
  --deepseek-ocr-force-prompt-eos \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --debug-artifacts \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-smoke \
  --summary unlimited-ocr-portable/analysis/summaries/SUMMARY-parity-artifacts-smoke.md
```

The recorded artifact smoke found a raw leading newline token in llama.cpp
before the same visible `<|det|>` token emitted first by SGLang. Deeper runtime
inspection showed the newline came from the DeepSeek-OCR image-end marker, while
the remaining extra token in the earlier no-image-end artifact was forced EOS.
Combining no forced EOS with `--deepseek-ocr-no-image-end` gives exact SGLang
processor prefill parity on `sc-02` / `document_parsing`.

Runtime parity commands:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m analysis.uocr_harness.cli inspect-sglang-processor \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --image-mode gundam \
  --media-placement separate \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-smoke \
  --summary unlimited-ocr-portable/analysis/summaries/SUMMARY-runtime-parity-noimgend-noeos-smoke.md
```

Native SGLang `/generate` artifacts can capture input logprobs:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m analysis.uocr_harness.cli run-sglang \
  --start-server \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --max-tokens 1 \
  --debug-native-artifacts \
  --debug-top-logprobs 5 \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-onetok \
  --summary unlimited-ocr-portable/analysis/summaries/SUMMARY-parity-artifacts-native-onetok.md
```

The one-token native artifact comparison aligns first token `<|det|>` and
first-output top-k overlap 1.000. Hidden-state return was also validated in an
isolated `/tmp` run; SGLang returned summarized hidden states with shape
`[1, 1517, 1280]`. The patched native runner can now also write llama.cpp
output-embedding summaries with `--debug-output-embeddings`; the one-token
smoke captured a 1280-wide prefill-last embedding and one generated-token
embedding in
`analysis/summaries/SUMMARY-parity-artifacts-output-embeddings-onetok.md`. The
restored-reference 20-row exact-prefill target run reaches 10 pass / 20 with
average similarity 0.512, a slight target-set improvement over the prior Q4
target setting. It is still not production-ready, so the remaining blocker is
later-token runtime drift rather than prompt token layout.

Generation-step artifact comparison is now available for native SGLang
`/generate` artifacts and llama.cpp `LLAMA_UOCR_PARITY_DUMP` artifacts:

```sh
uv run --project unlimited-ocr-portable uocr-harness compare-generation-artifacts \
  --results /tmp/uocr-step-results \
  --manifest unlimited-ocr-portable/results/manifest.jsonl \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-64tok \
  --summary unlimited-ocr-portable/analysis/summaries/SUMMARY-generation-steps-noimgend-noeos-64tok.md
```

The recorded Q4 exact-prefill run matches SGLang for the first three generated
tokens, then diverges on the first bbox coordinate: SGLang selects token `6207`
(`91`), while Q4 selects token `6152` (`92`). Disabling the SWA experiment does
not change this first divergence. Q5_K_M, Q6_K, and BF16 diverge earlier at
step 1 by ranking `aside` over SGLang's `header`.

The historical full exact-prefill/no-image-end Q4 run without the older CLI SWA
experiment was worse than the old full baseline: 49 / 104 passes, 5 empty rows,
27 repetition rows, and average similarity 0.671 in
`analysis/summaries/SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`.
The current exact-prefill/no-image-end R-SWA variant ties 56 / 104 passes and
raises average similarity to 0.719, but still has 5 empty rows. It is an
alternate diagnostic path, not production parity.

The current practical Q4 full command is:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --max-tokens 8192 \
  --ctx-size 32768 \
  --candidate-engine llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full \
  --deepseek-ocr-mode gundam \
  --deepseek-ocr-force-prompt-eos \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --force
```

The BF16 R-SWA full-run quality ceiling has the best current pass count:
61 passes and average similarity 0.684 in
`analysis/summaries/SUMMARY-uocr-rswa-bf16-eos-origin-ngram-default-full.md`.

## Portable Client Demo

The demo client now lives in `src/baidu_unlimited_ocr_portable/` and is exposed
by the root uv project as `baidu-uocr-client`. It does not load SGLang, PyTorch,
Transformers, or the Baidu custom SGLang wheel. Python launches the patched
native `llama-uocr-parity` binary as a subprocess, streams generated stdout into
the UI, parses `<|det|>` / `<|ref|>` markers, and renders bounding-box overlays.

Default profile:

- `llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full`
- 54 / 104 pass, 0 empty rows, 19 repetition rows, average similarity 0.678.

The UI also exposes
`llamacpp-q4_k_m-uocr-rswa-noimgend-noeos-full` as an experimental
profile. It reached average similarity 0.719 but produced 5 empty rows, so it
is not the default.

Run a short native smoke from the repository root:

```sh
uv run --project unlimited-ocr-portable baidu-uocr-client \
  --smoke --image dataset/sc-02.png --max-tokens 64
```

Launch the UI:

```sh
uv run --project unlimited-ocr-portable baidu-uocr-client \
  --host 127.0.0.1 --port 7861
```

The WSL2 smoke executed on 2026-06-27 produced non-empty `<|det|>` OCR output
from both exposed profiles. The Gradio endpoint responded at
`http://127.0.0.1:7861`, and the PDF/overlay smoke rendered 6 pages from
`dataset/chinese-paper.pdf`, parsed 1 marker box, and generated a preview.

## Output Layout

```text
results/
  manifest.jsonl
  prepared/
  inspection/preprocessing.jsonl
  inspection/sglang_processor.jsonl
  artifacts/reference/sglang/<case_id>/<profile>.sglang.json
  artifacts/reference/sglang-processor/<case_id>/<profile>.processor.json
  artifacts/reference/sglang-native/<case_id>/<profile>.sglang-native.json
  artifacts/candidate/<strategy>/<case_id>/<profile>.llamacpp.json
  reference/sglang/<case_id>/<profile>.json
  candidate/llamacpp-q4_k_m/<case_id>/<profile>.json
  candidate/<strategy>/<case_id>/<profile>.json
  compare/metrics.csv
analysis/
  summaries/SUMMARY.md
  summaries/SUMMARY-*.md
  uocr_harness/
src/
  baidu_unlimited_ocr_portable/
```

See `TEST-PROCEDURE.md` for the full reproducible procedure, plus
`docs/LINUX.md` and `docs/WINDOWS.md` for platform-specific notes.

## Safetensors Header Probe

Build:

```sh
g++ -std=c++17 -O2 -Wall -Wextra safetensors_probe.cpp -o safetensors_probe
```

Run:

```sh
./safetensors_probe ../unlimited-ocr/model-00001-of-000001.safetensors
```

The probe reads only the safetensors JSON header and summarizes tensor-name
prefixes. It is intended to verify architecture shape without depending on
Python, PyTorch, or full model loading.
