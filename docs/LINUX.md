# Linux / WSL2 CUDA Validation

These commands are tested for Ubuntu 24.04 / WSL2 with NVIDIA CUDA available.
Run them from `/home/ubuntu/projects/unlimited-ocr`.

The full executed validation procedure is documented in
`../TEST-PROCEDURE.md`. This Linux document keeps the platform-specific setup
and command notes.

## Current R-SWA Status

The local llama.cpp branch now includes PR #24975 style reference-SWA behavior
for DeepSeek-OCR/Unlimited-OCR. The older CLI KV-pruning SWA experiment is
disabled by default and only runs when
`LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE=1` is explicitly set.

Latest full-matrix results are summarized in
`../analysis/summaries/SUMMARY-uocr-rswa-executive.md`:

- Q4_K_M R-SWA default: 54 / 104 pass, 0 empty, 19 repetition, average
  similarity 0.678.
- Q4_K_M exact-prefill/no-image-end R-SWA: 56 / 104 pass, 5 empty, average
  similarity 0.719.
- BF16 R-SWA: 61 / 104 pass, 0 empty, 18 repetition, average similarity 0.684.

This is not production parity. R-SWA improves the BF16 quality ceiling, but the
practical Q4 default is slightly worse than the older CLI-prune baseline.

## Prerequisites

- `uv`, `git`, `gh`, CUDA toolkit, and NVIDIA driver are available.
- The local SGLang environment is installed in `.venv`.
- llama.cpp is checked out at `thirdparty/llama.cpp`.
- GGUF files are present in `thirdparty/uocr-gguf`.

Verify:

```sh
nvidia-smi
uv run --project unlimited-ocr-portable uocr-harness --help
thirdparty/llama.cpp/build/bin/llama-mtmd-cli --version
```

## Repair / Verify SGLang Reference Environment

The reference environment uses Baidu's bundled custom SGLang wheel, not a
generic SGLang install. If `.venv` drifts, realign it with:

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

Verify the ABI-sensitive imports:

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

## Build llama.cpp With CUDA

Use the custom branch already tested in this workspace unless a new validation
run intentionally updates it:

```text
thirdparty/llama.cpp branch: uocr-deepseek-ocr-parity
f3e5dcccf deepseek2-ocr: add Unlimited-OCR R-SWA parity
7b0ec28 mtmd-cli: dump OCR output embedding summaries
48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner
8fbbd5b mtmd-cli: add OCR sampling parity controls
3ebff83 mtmd: add Unlimited-OCR gundam grid parity
9d5d882 model : Add label for LFM2.5-230M (#25008)
```

```sh
uv tool run cmake -B thirdparty/llama.cpp/build \
  -S thirdparty/llama.cpp \
  -DGGML_CUDA=ON \
  -DCMAKE_BUILD_TYPE=Release

uv tool run cmake --build thirdparty/llama.cpp/build -j \
  --target llama-mtmd-cli llama-uocr-parity llama-server
```

If the build directory already exists and `cmake` is not on PATH, rebuild the
existing Makefile targets:

```sh
make -C thirdparty/llama.cpp/build llama-mtmd-cli llama-uocr-parity llama-server -j8
```

The current workspace includes opt-in llama.cpp patches for SGLang-style
DeepSeek-OCR gundam preprocessing, local-grid embedding composition,
SGLang-style no-repeat defaults, core reference-SWA masking, and native parity
artifact dumping through `llama-uocr-parity`. The stable baseline remains native
DeepSeek-OCR preprocessing unless `--deepseek-ocr-mode gundam` is passed.

Runtime parity artifacts are also available from the portable harness:

- `inspect-sglang-processor` records the SGLang processor/template input IDs.
- `run-sglang --debug-native-artifacts` records native `/generate` input
  logprobs and first-output top-k data.
- `compare-runtime-parity` compares SGLang processor artifacts with llama.cpp
  native `LLAMA_UOCR_PARITY_DUMP` artifacts.
- `compare-generation-artifacts` compares native SGLang generated token IDs and
  top-k lists against llama.cpp generation-step artifacts.
- `run-llamacpp --debug-output-embeddings` records opt-in llama.cpp output
  embedding summaries in the native parity artifact.

Use the repo `.venv` for SGLang processor/native artifact commands:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m analysis.uocr_harness.cli inspect-sglang-processor \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --force
```

## Prepare Dataset

```sh
uv run --project unlimited-ocr-portable uocr-harness prepare
```

The harness normalizes all image inputs to PNG, applies EXIF orientation, and
renders PDF pages at 300 DPI. PDF page order is preserved.

## Run llama.cpp Candidate

Small smoke run:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --limit 1 \
  --max-tokens 256
```

Full candidate run:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp
```

Current practical Q4 full candidate run:

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

Exact-prefill diagnostic target run:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4,chinese-paper-page-0001-2200885e,chinese-paper-page-0002-3d10e38a,sc-02-45a8efac,upside-left-9e645a2a \
  --profiles grounding,plain_text,ocr_boxes,document_parsing \
  --candidate-engine llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --deepseek-ocr-no-repeat-ngram \
  --deepseek-ocr-prefill-aware-swa \
  --deepseek-ocr-decode-window 128 \
  --deepseek-ocr-no-image-end \
  --max-tokens 8192 \
  --ctx-size 32768
```

This exact-prefill run validates tokenizer/template parity and reached
10 pass / 20 with average similarity 0.512 on the restored-reference target
set. It slightly improves the prior Q4 target average of 0.502, but it is still
not production-ready. The current practical full setting is the 104-row
`llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full` run.

Useful candidate strategy switches:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --candidate-engine llamacpp-q6_k \
  --model thirdparty/uocr-gguf/Unlimited-OCR-Q6_K.gguf \
  --quantization Q6_K \
  --repeat-penalty 1.05 \
  --image-min-tokens 1024 \
  --image-max-tokens 2048
```

Use `--candidate-engine` to keep strategy outputs separate under
`results/candidate/<strategy>/`.

Inspect preprocessing parity:

```sh
uv run --project unlimited-ocr-portable uocr-harness inspect-preprocessing
```

Run the exact DeepSeek-OCR gundam candidate smoke:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp \
  --case-id sc-02-45a8efac \
  --profiles document_parsing \
  --candidate-engine llamacpp-q4_k_m-gundam-exact-prefix-tight \
  --deepseek-ocr-mode gundam \
  --media-placement prefix-tight \
  --max-tokens 1024
```

Run the native artifact smoke:

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

## Run Portable Client Demo

The demo under `unlimited-ocr-portable/src/baidu_unlimited_ocr_portable` is the
current interactive candidate UI. It calls the patched native
`llama-uocr-parity` binary directly and has no SGLang/PyTorch/Transformers
dependency.

Short smoke:

```sh
uv run --project unlimited-ocr-portable baidu-uocr-client \
  --smoke --image dataset/sc-02.png --max-tokens 64
```

Launch:

```sh
uv run --project unlimited-ocr-portable baidu-uocr-client \
  --host 127.0.0.1 --port 7861
```

Open `http://127.0.0.1:7861`.

Default paths:

- `thirdparty/llama.cpp/build/bin/llama-uocr-parity`
- `thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf`
- `thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf`

Override with `UOCR_LLAMA_BIN`, `UOCR_MODEL`, and `UOCR_MMPROJ` when testing
another build or model location.

## Run llama-server Candidate

The harness can also start `llama-server` and call its native multimodal
`/completion` endpoint:

```sh
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp-server \
  --candidate-engine llamacpp-server-q4_k_m
```

The OpenAI-compatible chat endpoint initially failed on this build because the
MTMD media marker was not preserved for the DeepSeek-OCR template. The harness
therefore uses `/completion` with `prompt_string` and `multimodal_data`.

## Run SGLang BF16 Reference

If a SGLang server is already listening at `127.0.0.1:10000`:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang
```

To let the harness start and stop SGLang:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server
```

Reference outputs are written to `unlimited-ocr-portable/results/reference/sglang`.

On this RTX 5090 / SM120 WSL2 machine the harness defaults SGLang to
`--attention-backend flashinfer`. FA3 failed here because the custom SGLang build
restricts FA3 to SM80-SM90. Override only when validating on hardware where the
backend is supported:

```sh
uv run --project unlimited-ocr-portable --python .venv/bin/python \
  uocr-harness run-sglang --start-server --attention-backend fa3
```

## Compare and Summarize

```sh
uv run --project unlimited-ocr-portable uocr-harness compare
```

The comparator writes:

- `unlimited-ocr-portable/results/compare/metrics.csv`
- `unlimited-ocr-portable/analysis/summaries/SUMMARY.md`

The status categories, expected counts, and interpretation rules are documented
in `../TEST-PROCEDURE.md`.

## Known Linux CUDA Notes

- The latest decision result is in
  `unlimited-ocr-portable/analysis/summaries/SUMMARY-uocr-rswa-executive.md`.
- In the latest Q4_K_M R-SWA full run, llama.cpp completed all 104 rows with
  zero empty outputs, 54 automated passes, 19 repetition rows, and average
  similarity 0.678.
- BF16 R-SWA is the current pass-count ceiling: 61 automated passes, 18
  repetition rows, and average similarity 0.684. It is slower and still not
  production parity.
- The patched DeepSeek-OCR gundam path now composes local crop embeddings into
  the SGLang-style local grid. The `sc-02` / `document_parsing` smoke passed at
  similarity 0.998 with matching bbox marker counts.
- The best five-case, four-profile Q4 target run passed 9 / 20 rows and still
  had 6 repetition failures.
- Stable unpatched llama-server is not sufficient for current parity. Use the
  local custom llama.cpp branch plus focused patches unless they are upstreamed.
  `llama-server` has the shared MTMD grid patch, while the no-repeat,
  forced-EOS, and debug controls were validated through `llama-mtmd-cli` /
  `llama-uocr-parity`.
- `llama-uocr-parity` is the current native debug runner. It showed the patched
  forced-EOS candidate emits raw newline token `201` before the same visible
  `<|det|>` token SGLang emits first. Later runtime inspection showed exact
  prefill parity requires no forced EOS plus `--deepseek-ocr-no-image-end`.
- Exact-prefill/no-image-end Q4 is diagnostic only. The current R-SWA full
  variant tied 56 automated passes and average similarity 0.719, but still had
  5 empty rows.
- Generation-step comparison on `sc-02` / `document_parsing` shows Q4 matches
  SGLang through `<|det|>header [` and then diverges on the first bbox
  coordinate (`91` vs `92`). Q5_K_M, Q6_K, and BF16 diverge earlier at
  `header` vs `aside`.
- Output-embedding smoke on `sc-02` / `document_parsing` captured SGLang hidden
  shape `[1, 1517, 1280]` and llama.cpp prefill/generation output embeddings
  with width 1280.
- llama.cpp may warn that CUDA flash attention is unsupported for this graph.
- llama.cpp may warn that some CLIP permute operators are not CUDA-backed.
- These warnings do not block correctness validation, but they do mean
  performance should be measured before using the runtime in production.
