# Linux / WSL2 CUDA Validation

These commands are tested for Ubuntu 24.04 / WSL2 with NVIDIA CUDA available.
Run them from `/home/ubuntu/projects/unlimited-ocr`.

The full executed validation procedure is documented in
`../TEST-PROCEDURE.md`. This Linux document keeps the platform-specific setup
and command notes.

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
SGLang-style no-repeat defaults, prefill-aware SWA experiments, and native
parity artifact dumping through `llama-uocr-parity`. The stable baseline remains
native DeepSeek-OCR preprocessing unless `--deepseek-ocr-mode gundam` is passed.

Runtime parity artifacts are also available from the portable harness:

- `inspect-sglang-processor` records the SGLang processor/template input IDs.
- `run-sglang --debug-native-artifacts` records native `/generate` input
  logprobs and first-output top-k data.
- `compare-runtime-parity` compares SGLang processor artifacts with llama.cpp
  native `LLAMA_UOCR_PARITY_DUMP` artifacts.
- `compare-generation-artifacts` compares native SGLang generated token IDs and
  top-k lists against llama.cpp generation-step artifacts.

Use the repo `.venv` for SGLang processor/native artifact commands:

```sh
PYTHONPATH=unlimited-ocr-portable uv run --no-project --python .venv/bin/python \
  -m uocr_harness.cli inspect-sglang-processor \
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

Current best full candidate run:

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
not production-ready. The current best full setting remains the 104-row
`llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full` run.

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
  --summary unlimited-ocr-portable/SUMMARY-parity-artifacts-smoke.md
```

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
- `unlimited-ocr-portable/SUMMARY.md`

The status categories, expected counts, and interpretation rules are documented
in `../TEST-PROCEDURE.md`.

## Known Linux CUDA Notes

- The current full comparison result is in `unlimited-ocr-portable/SUMMARY.md`.
- In the latest patched Q4_K_M full run, llama.cpp completed all 104 rows with
  zero empty outputs, 56 automated passes, 17 repetition rows, and average
  similarity 0.688.
- Full BF16 GGUF was retested as a quality ceiling and did not beat Q4:
  54 automated passes, 27 repetition rows, and average similarity 0.649.
- The patched DeepSeek-OCR gundam path now composes local crop embeddings into
  the SGLang-style local grid. The `sc-02` / `document_parsing` smoke passed at
  similarity 0.998 with matching bbox marker counts.
- The best five-case, four-profile Q4 target run passed 9 / 20 rows and still
  had 6 repetition failures.
- Stable unpatched llama-server is not sufficient for current parity. Use the
  local custom llama.cpp branch plus focused patches unless they are upstreamed.
  `llama-server` has the shared MTMD grid patch, but the no-repeat/SWA/forced
  EOS controls were validated through `llama-mtmd-cli`.
- `llama-uocr-parity` is the current native debug runner. It showed the patched
  forced-EOS candidate emits raw newline token `201` before the same visible
  `<|det|>` token SGLang emits first. Later runtime inspection showed exact
  prefill parity requires no forced EOS plus `--deepseek-ocr-no-image-end`.
- Exact-prefill/no-image-end Q4 is diagnostic only: the full 104-row run
  regressed to 49 automated passes, 5 empty rows, 27 repetition rows, and
  average similarity 0.671.
- Exact-prefill/no-image-end/SWA128 ties the 56-pass baseline and improves
  average similarity to 0.717, but still has 5 empty rows and 17
  low-similarity rows.
- Generation-step comparison on `sc-02` / `document_parsing` shows Q4 matches
  SGLang through `<|det|>header [` and then diverges on the first bbox
  coordinate (`91` vs `92`). Q5_K_M, Q6_K, and BF16 diverge earlier at
  `header` vs `aside`.
- llama.cpp may warn that CUDA flash attention is unsupported for this graph.
- llama.cpp may warn that some CLIP permute operators are not CUDA-backed.
- These warnings do not block correctness validation, but they do mean
  performance should be measured before using the runtime in production.
