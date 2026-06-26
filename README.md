# Unlimited-OCR Portable

This directory contains local portable-runtime experiments and the validation
harness for comparing:

- SGLang BF16 reference output from WSL2/Linux.
- llama.cpp/GGUF candidate output from Linux or Windows.

Generated harness outputs live under `results/` and are ignored by git. The
portable summary is written to `SUMMARY.md`.

Use `TEST-PROCEDURE.md` as the canonical runbook for the validation pass that
produced the current summary.

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

The current full WSL2 run is summarized in `SUMMARY.md`. At the time of writing,
SGLang BF16 completed all 104 reference rows. The best patched Q4_K_M candidate
completed all 104 rows with zero empty outputs, 56 automated passes, 17
repetition rows, and average similarity 0.688. This is materially better than
the native baseline but still not production-ready.

For the exact commands, environment repair steps, expected file counts, and
interpretation rules, see `TEST-PROCEDURE.md`.

Follow-up targeted strategy summaries:

- `SUMMARY-runtime-parity-noimgend-noeos-smoke.md`
- `SUMMARY-parity-artifacts-native-onetok.md`
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
SGLang-style no-repeat defaults, forced prompt EOS, prefill-aware SWA128, and
diagnostic image-end/min-new-token switches.

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
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-smoke.md
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
  -m uocr_harness.cli inspect-sglang-processor \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --image-mode gundam \
  --media-placement separate \
  --force

uv run --project unlimited-ocr-portable uocr-harness compare-runtime-parity \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-smoke \
  --summary unlimited-ocr-portable/SUMMARY-runtime-parity-noimgend-noeos-smoke.md
```

Native SGLang `/generate` artifacts can capture input logprobs:

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

uv run --project unlimited-ocr-portable uocr-harness compare-artifacts \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --reference-engine sglang-native \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-onetok \
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-native-onetok.md
```

The one-token native artifact comparison aligns first token `<|det|>` and
first-output top-k overlap 1.000. Hidden-state return was also validated in an
isolated `/tmp` run; SGLang returned summarized hidden states with shape
`[1, 1517, 1280]`. The restored-reference 20-row exact-prefill target run
reaches 10 pass / 20 with average similarity 0.512, a slight target-set
improvement over the prior Q4 target setting. It is still not production-ready,
so the remaining blocker is later-token runtime drift rather than prompt token
layout.

The current best full candidate is:

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

The BF16 full-run quality ceiling did not beat Q4 on the full matrix: 54 passes
and average similarity 0.649 in
`SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md`.

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
SUMMARY.md
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
