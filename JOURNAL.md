# Unlimited-OCR Portable Runtime Journal

Date: 2026-06-26 Asia/Manila local environment. Upstream timestamps below are
UTC where they came from GitHub or Hugging Face.

## Objective

Investigate whether Unlimited-OCR can reasonably get a Python-free C/C++
runtime, focusing first on Linux + NVIDIA CUDA while keeping future Windows,
macOS, ROCm, and oneAPI portability in mind.

The practical output is a yes/no recommendation for a one-person effort, not an
unbounded "anything is possible" answer.

## Current Decision

Yes, a practical C/C++ runtime is feasible for a one-person effort if we build
on an existing runtime, especially llama.cpp/ggml with GGUF model artifacts.
The near-term target should be:

1. Linux + NVIDIA CUDA.
2. llama.cpp multimodal runtime.
3. Community GGUF weights plus `mmproj`.
4. Validation against the current SGLang output for the same images and prompts.

No, a from-scratch pure C/C++ runtime covering CUDA, MLX, ROCm, and oneAPI is
not reasonable for one person. It would require reimplementing too much model,
vision, tokenizer, quantization, KV-cache, scheduling, and backend kernel code.

## Local Model Evidence

Local files inspected:

- `unlimited-ocr/config.json`
- `unlimited-ocr/processor_config.json`
- `unlimited-ocr/modeling_unlimitedocr.py`
- `unlimited-ocr/deepencoder.py`
- `unlimited-ocr/modeling_deepseekv2.py`
- `unlimited-ocr-github/infer.py`
- `README.md`

Architecture facts from local source:

- Model class: `UnlimitedOCRForCausalLM`.
- Vision path: SAM-style ViT-B image encoder plus CLIP-L-like ViT, followed by a
  linear projector from 2048 to 1280.
- Text path: DeepSeek-V2 style decoder, but configured with plain MHA
  (`use_mla=false`) and MoE after layer 0.
- Decoder size: 12 layers, hidden size 1280, 10 attention heads, vocab 129280,
  max position 32768.
- MoE: 64 routed experts, 2 shared experts, 6 experts per token,
  `first_k_dense_replace=1`.
- Runtime behavior includes image token insertion, dynamic crop mode
  (`gundam`), non-crop `base` mode, multi-image base mode, custom no-repeat
  n-gram processing, and a custom sliding-window/ring-cache path.

I added a small C++ safetensors header probe under `unlimited-ocr-portable/`.
It reads only the safetensors JSON header and does not load weights through
Python or PyTorch.

Probe command:

```sh
g++ -std=c++17 -O2 -Wall -Wextra safetensors_probe.cpp -o /tmp/uocr_safetensors_probe
/tmp/uocr_safetensors_probe ../unlimited-ocr/model-00001-of-000001.safetensors
```

Probe result:

```text
file_size_bytes: 6672547120
header_bytes: 334632
top_level_entries: 2711
tensor_entries: 2710

category_counts:
  lm_head                        1
  text.attention                 48
  text.dense_mlp                 3
  text.embed_tokens              1
  text.final_norm                1
  text.layer_norms               24
  text.moe.routed_experts        2112
  text.moe.router                11
  text.moe.shared_experts        33
  vision.clip_l                  293
  vision.projector               2
  vision.sam_vit_b               179
  vision.special_embeddings      2

dtype_counts:
  BF16     2710
```

This confirms that the local weights contain both the custom vision stack and
the MoE decoder in one BF16 safetensors shard. It also confirms that a C/C++
runtime is not just a text-decoder problem.

## Upstream Evidence

Read-only GitHub and Hugging Face checks were used. No GitHub or Hugging Face
write operations were performed.

### Baidu issue 7: vLLM support

URL: https://github.com/baidu/Unlimited-OCR/issues/7

State on read: open. A contributor commented on 2026-06-23 that only the
SGLang wheel existed at that time, and vLLM support was already under
development.

### Baidu issue 9: quantized weights

URL: https://github.com/baidu/Unlimited-OCR/issues/9

State on read: open. A contributor commented on 2026-06-23 that there was no
official plan yet to release quantized weights, but they might consider it
depending on demand.

### Baidu issue 27: deployment blockers

URL: https://github.com/baidu/Unlimited-OCR/issues/27

State on read: open, created 2026-06-25. The issue summarizes current adoption
blockers:

- Bundled custom SGLang wheel is easy to miss.
- Standard SGLang installs do not work reliably for this model.
- `UnlimitedOCRForCausalLM` registration failures are still a real issue.
- No official Docker/container image yet.
- vLLM support is not merged in Baidu's repo yet.
- Official quantized weights are absent, though community quantizations exist.

Important maintainer comments:

- Official Docker image or Dockerfile is planned "as soon as possible".
- vLLM support is tracked at `vllm-project/vllm#46564` and was expected within
  the week if things went smoothly.
- Community INT4, AWQ, and GGUF quantizations are acknowledged.
- The team is aware that the custom SGLang wheel makes setup difficult and is
  working on a more robust SGLang environment.

Important community follow-up:

- `--trust-remote-code` alone does not fully explain issue #12. The startup log
  shows SGLang ignoring import errors for internal `unlimited_ocr` model and
  processor modules. That points to a wheel-side registration/import problem.
- Setting OpenAI API `max_tokens=32768` is unsafe unless image-token headroom is
  considered.
- PDF helper scripts may leak temp PNGs and can scramble page order if sorting
  is applied to PDF pages.
- README text has a `kernels==0.9.0` versus `kernels==0.11.7` mismatch.

This issue strengthens the conclusion that SGLang is currently not the right
foundation for a Python-free portable runtime. It is useful as a correctness
oracle, not as the portable path.

### Baidu PR 29

URL: https://github.com/baidu/Unlimited-OCR/pull/29

State on read: open and mergeable. It adds only two lines to `infer.py`:

- `max_tokens` guard.
- `--trust-remote-code` in SGLang launch.

Given issue #27's later comment, this PR should be treated as partial mitigation
only. It does not prove that SGLang startup is solved for all users.

### vLLM PR 46564

URL: https://github.com/vllm-project/vllm/pull/46564

State on read: open, mergeable true, merge state blocked. It adds native
Unlimited-OCR support across many vLLM components, including:

- `vllm/model_executor/models/unlimited_ocr.py`
- `vllm/transformers_utils/processors/unlimited_ocr.py`
- DeepSeek-V2 model changes
- attention backend changes
- KV-cache manager changes
- an R-SWA cache/mask path

The PR description says Unlimited-OCR reuses the DeepSeek-OCR vision stack but
uses a DeepSeek-V2 MoE language backbone with plain MHA. It also states that
Unlimited-OCR uses reference sliding-window attention where prompt/image tokens
remain globally visible while generated tokens use a recent-token sliding
window.

This is strong evidence that a correct optimized runtime needs more than a
basic transformer port. KV-cache semantics and multimodal image-token layout are
core requirements.

### llama.cpp PR 17400

URL: https://github.com/ggml-org/llama.cpp/pull/17400

State on read: merged on 2026-03-26. It added DeepSeek-OCR multimodal support
to llama.cpp/ggml, including:

- GGUF conversion changes.
- DeepSeek2 architecture/tensor mapping updates.
- multimodal `mtmd` DeepSeek-OCR model code.
- CLIP/vision changes.
- tests and sample extracted outputs.

This is the strongest C/C++ runtime foundation found so far. It does not prove
that Unlimited-OCR works out of the box, but Unlimited-OCR's public GGUF models
claim to rely on this DeepSeek-OCR-aware llama.cpp path.

### Hugging Face model ecosystem

The base model `baidu/Unlimited-OCR` is a Transformers/custom-code model with
about 3.336B BF16 parameters and local safetensors size around 6.67 GB.

Community artifacts found through Hugging Face model search:

- `sahilchachra/Unlimited-OCR-GGUF`
- `sahilchachra/Unlimited-OCR-AWQ`
- `sahilchachra/Unlimited-OCR-NVFP4`
- multiple MLX quantizations
- OpenVINO/ONNX experiments

The GGUF repo is the most relevant to a C/C++ path. It provides language-model
GGUF quantizations plus `mmproj-Unlimited-OCR-F16.gguf`. Its model card says a
DeepSeek-OCR-aware llama.cpp build is required and gives `llama-mtmd-cli` and
`llama-server` examples. The referenced DeepSeek-OCR llama.cpp PR is now merged,
although the card may have been written before that merge was universally
available in packages.

The AWQ repo is useful for low-VRAM Python/Transformers or vLLM-style stacks,
but it is not a pure C/C++ runtime path.

## Runtime Work Required

A correct C/C++ runtime must implement or reuse all of the following:

- Tokenizer and chat/template behavior, including special image token IDs.
- Image loading, EXIF correction, RGB conversion, padding, normalization, and
  dynamic crop tiling.
- SAM-style ViT-B encoder.
- CLIP-L-like ViT encoder that consumes SAM patch embeddings.
- Feature concatenation, newline/view separator embeddings, and linear
  projector.
- Image token sequence construction for base, gundam, and multi-image modes.
- DeepSeek-V2/Llama-like decoder with MoE routing.
- Efficient expert dispatch for 64 routed experts and 2 shared experts.
- BF16 and quantized weight loading.
- KV cache with Unlimited-OCR/DeepSeek-OCR attention behavior.
- Long-output safeguards, including no-repeat n-gram behavior or an equivalent
  repetition-control path.
- Output post-processing for grounding/detection markers if the application
  needs markdown, crops, or bounding boxes.

Building this from scratch would be a model runtime project, not just an OCR
wrapper.

## Feasible Path

First milestone result:

1. Keep SGLang running as the correctness oracle.
2. Use llama.cpp/ggml as the C/C++ runtime base.
3. Use `sahilchachra/Unlimited-OCR-GGUF` or another community GGUF with the
   matching `mmproj` file.
4. Validate `llama-mtmd-cli` on one image using `document parsing.` and
   `<|grounding|>Convert the document to markdown.` prompts.
5. Compare output against SGLang for the same input, prompt, temperature, and
   max-token budget.
6. If llama.cpp rejects the Unlimited-OCR GGUF metadata, patch metadata or
   llama.cpp's DeepSeek-OCR mapping rather than writing a new runtime.
7. Once Linux + CUDA works, package a tiny C/C++ CLI and server wrapper around
   llama.cpp.

The basic Linux + CUDA load/generate test passed on this machine.

## llama.cpp Smoke Test

Local setup:

- llama.cpp cloned to `thirdparty/llama.cpp`.
- llama.cpp commit: `9d5d882`.
- Built with CUDA enabled using CMake from `uv tool run cmake`.
- CUDA toolkit detected: 12.9.86.
- Native GPU architecture detected by CMake: `120a-real`.
- GPU: NVIDIA GeForce RTX 5090, about 32 GB VRAM.
- Built targets: `llama-mtmd-cli` and `llama-server`.

Downloaded model files:

- `thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf` (about 1.9 GB)
- `thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf` (about 775 MB)

Smoke-test command:

```sh
thirdparty/llama.cpp/build/bin/llama-mtmd-cli \
  -m thirdparty/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf \
  --mmproj thirdparty/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf \
  --image unlimited-ocr/assets/Unlimited-OCR.png \
  -p 'document parsing.' \
  --chat-template deepseek-ocr \
  --temp 0 \
  --top-k 1 \
  -n 128 \
  -c 4096 \
  -ngl all \
  --log-verbosity 2
```

Observed warnings:

- Context was limited to 4096 for the smoke test, below the model's 32768
  training context.
- CUDA flash attention was not supported for this graph on `CUDA0`; llama.cpp
  continued with higher memory usage.
- The CLIP graph had unsupported CUDA backend permute ops, so performance may be
  suboptimal.

Observed output:

```text
<|/det|>image [14, 37, 432, 999]<|/det|>
<|det|>image [458, 33, 984, 999]<|/det|>
```

Conclusion from smoke test:

- The C++/CUDA path is not hypothetical. It can load a community Unlimited-OCR
  GGUF and `mmproj`, run multimodal preprocessing, and generate output.
- The first feasibility answer is therefore a stronger yes for Linux + NVIDIA
  CUDA using llama.cpp/ggml.
- The warnings mean this is not yet a polished production runtime. Performance
  and output quality still need validation on document pages.

Success criteria for a production-quality first milestone:

- No Python process in the serving/inference path.
- Builds on Ubuntu 24.04 with CUDA enabled.
- Loads GGUF language model plus `mmproj`.
- Accepts one image and returns OCR/markdown text.
- Output is qualitatively close to SGLang on at least a small fixed image set.
- Documents exact llama.cpp commit, model files, quantization, prompt, context,
  and generation settings.

## Non-goals For Now

- Do not attempt ROCm, oneAPI, MLX, Windows packaging, or macOS packaging until
  Linux + CUDA has a verified llama.cpp path.
- Do not build a custom inference engine from scratch.
- Do not depend on SGLang for the portable runtime. It remains useful for
  comparison only.
- Do not treat vLLM as a Python-free answer. It is a useful upstream reference
  and production server option, but not the C/C++ runtime target.

## Next Concrete Step

The first quality-validation pass is complete. The next engineering step is to
retest higher-quality GGUF variants or fix the llama.cpp prompt/template path
before packaging the portable runtime.

## 2026-06-26 Quality Validation Pass

The reproducible procedure for this pass is documented in
`unlimited-ocr-portable/TEST-PROCEDURE.md`. The current machine-readable metrics
and Markdown result summary are under `unlimited-ocr-portable/results/compare/`
and `unlimited-ocr-portable/SUMMARY.md`.

Dataset:

- Source directory: `/home/ubuntu/projects/unlimited-ocr/dataset`.
- Prepared manifest: `unlimited-ocr-portable/results/manifest.jsonl`.
- Case count: 26 prepared image/PDF-page cases.
- Prompt profiles per case: `grounding` and `plain_text`.
- Total comparison rows: 52.
- Token budget: 8192 max generated tokens for both reference and candidate.

SGLang reference environment repair:

- The repo-level `.venv` is the active SGLang environment.
- Installed the bundled custom wheel:
  `unlimited-ocr/wheel/sglang-0.0.0.dev11416+g92e8bb79e-py3-none-any.whl`.
- Realigned `.venv` to the custom wheel's dependency stack:
  `torch==2.9.1`, `torchaudio==2.9.1`, `torchvision==0.24.1`,
  `transformers==5.3.0`, `sglang-kernel==0.4.1`, `kernels==0.11.7`.
- Verified imports for `torch`, `sglang`, `sgl_kernel`, and
  `DeepseekOCRNoRepeatNGramLogitProcessor`.
- Isolated startup blockers found and fixed:
  - Public/newer `sglang-kernel` builds were ABI-incompatible with the custom
    SGLang wheel stack.
  - `--attention-backend fa3` failed on RTX 5090 / SM120 because this SGLang
    build restricts FA3 to SM80-SM90. The harness now defaults to
    `--attention-backend flashinfer` and exposes it as a CLI option.
  - FlashInfer JIT needed `ninja` on `PATH`. The harness now prepends the
    selected SGLang interpreter's `bin` directory to the server process `PATH`.

SGLang reference result:

- Full run completed: 52 / 52 result JSON files.
- Request errors: 0.
- Empty normalized outputs: 0.
- Average elapsed per row in harness metrics: 6239 ms.
- Average GPU memory after request while server remained loaded: 31674 MB.
- Server log ended with a graceful shutdown.

llama.cpp Q4_K_M candidate result:

- Full run completed: 52 / 52 result JSON files.
- Process errors: 0.
- Automated status counts:
  - `candidate_empty`: 41
  - `candidate_repetition`: 7
  - `bbox_count_mismatch`: 2
  - `pass`: 2
- Comparable non-empty text pairs: 11 / 52.
- Average text similarity across comparable pairs: 0.550.
- Average candidate elapsed per row: 2883 ms.
- Average candidate GPU-after-request snapshot: 2223 MB. This is not peak
  VRAM because the CLI process exits per row; spot checks during active
  llama.cpp generation showed much higher transient VRAM, around 9-12 GB.
- Rows with greater than 30% bbox-count delta: 50 / 52.

Current quality decision:

- Q4_K_M is not production-ready with the current llama.cpp invocation. The main
  failure mode is successful process exit with empty normalized output; the
  non-empty failures show repetition and bbox marker drift.
- This does not yet prove llama.cpp is the wrong runtime. It proves this exact
  Q4_K_M artifact plus prompt/template/run configuration is not sufficient for
  production packaging.

Current packaging decision:

- No custom llama-server fork is justified yet. A recent DeepSeek-OCR-capable
  llama.cpp build can load the community Unlimited-OCR GGUF and `mmproj`; the
  next blockers are output quality and prompting/runtime behavior, not evidence
  of custom GGUF metadata that requires a fork.
- `llama-server` can remain an independently invoked stable/pinned component
  for experiments and simple service packaging.
- A small custom C++ wrapper over llama.cpp APIs is still the preferred product
  direction once quality is acceptable, because it can own stable image input
  handling, prompt/profile selection, structured JSON output, and Windows/Linux
  packaging without forking llama-server.

Next validation steps:

1. Retest Q5_K_M, Q6_K, and BF16 GGUF if available with the same harness.
2. Check whether llama.cpp needs a different prompt/profile, chat template, or
   image mode equivalent to SGLang's `gundam` configuration.
3. Add peak GPU memory capture for per-process llama.cpp runs; the current
   after-request snapshot underreports CLI peak memory.
4. Keep `SUMMARY.md` as the current result source and rerun `uocr-harness
   compare` after each candidate change.

## 2026-06-26 Follow-up Strategy Matrix

This section records the practical follow-up items from `prompts.md` after the
full Q4_K_M baseline failed quality validation.

Targeted case set used for strategy runs:

- `613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4`
  - Large browser screenshot; Q4 baseline was empty.
- `chinese-paper-page-0001-2200885e`
  - PDF page where Q4 baseline was non-empty but bbox counts drifted.
- `chinese-paper-page-0002-3d10e38a`
  - PDF page with both empty and high-repetition Q4 failures.
- `sc-02-45a8efac`
  - Screenshot with severe repetition in one profile.
- `upside-left-9e645a2a`
  - Rotated/preprocessing-sensitive image.

The target set has 5 cases and 2 profiles for most strategies, so each summary
normally contains 10 comparison rows.

### Strategy: Higher-Quality GGUFs

Action:

- Downloaded additional GGUFs from `sahilchachra/Unlimited-OCR-GGUF`:
  - `Unlimited-OCR-Q5_K_M.gguf`
  - `Unlimited-OCR-Q6_K.gguf`
  - `Unlimited-OCR-BF16.gguf`
- Kept the shared `mmproj-Unlimited-OCR-F16.gguf`.
- Added `--candidate-engine` and `--quantization` to the harness so each
  variant writes to a separate result directory.

Summaries:

- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`
- `SUMMARY-bf16.md`

Results:

- Q5_K_M: 6 empty, 3 repetition, 1 bbox mismatch, 0 pass.
- Q6_K: 6 empty, 2 repetition, 1 malformed marker, 1 bbox mismatch, 0 pass.
- BF16 GGUF: 5 empty, 3 repetition, 2 bbox mismatch, 0 pass.

Decision:

- The failure is not solved by moving from Q4_K_M to Q5_K_M, Q6_K, or BF16 GGUF
  on this targeted set. Because BF16 GGUF still fails, the primary blocker is
  more likely llama.cpp invocation/runtime/image-template behavior than
  quantization quality alone.

### Strategy: Repeat Penalty

Action:

- Added `--repeat-penalty` to `run-llamacpp`.
- Ran Q4_K_M with `--repeat-penalty 1.05`, matching the GGUF model-card tip for
  dense-page loops.

Summary:

- `SUMMARY-q4-rp105.md`

Results:

- 5 empty, 3 repetition, 2 bbox mismatch, 0 pass.
- Repetition penalty slightly changed some non-empty rows but did not fix the
  main empty-output or severe-loop pattern.

Decision:

- Repeat penalty is worth keeping as a runtime knob, but `1.05` is not enough
  to make Q4_K_M production-ready.

### Strategy: Prompt / Template Matrix

Action:

- Added model-card prompt profiles:
  - `ocr_boxes`: `<|grounding|>OCR this image.`
  - `document_parsing`: `document parsing.`
- Ran matching SGLang references for those profiles on the target cases.
- Ran Q4_K_M candidate with the same profiles.

Summary:

- `SUMMARY-q4-prompts.md`

Results:

- 0 empty rows, which proves prompt changes can avoid immediate blank output.
- 8 repetition rows, 1 malformed-marker row, 1 bbox mismatch, 0 pass.
- Average similarity dropped to 0.203.

Decision:

- Prompt changes are a real axis, but these two model-card prompts convert
  blank failures into long/repetitive failures. They are not a production fix.

### Strategy: llama-server Versus llama-mtmd-cli

Action:

- Added `run-llamacpp-server`.
- Initial OpenAI chat-completions request failed with HTTP 400 and server log:
  `number of media markers in text (0) does not match number of bitmaps (1)`.
- Patched the runner to use llama.cpp's native `/completion` endpoint with
  `prompt_string` and `multimodal_data`, using `/props` to read the server's
  MTMD media marker.

Summary:

- `SUMMARY-llamacpp-server-q4.md`

Results after the `/completion` fix:

- 7 empty, 1 repetition, 2 bbox mismatch, 0 pass.
- No HTTP 400 failures after switching to `/completion`.
- Server mode produces valid result JSON and can be used for future native
  serving tests, but it does not improve quality on the target set.

Decision:

- `llama-server` is enabled as a candidate strategy, but the OpenAI-compatible
  chat path is not sufficient for this model/build without careful media-marker
  handling. Native `/completion` works mechanically but is not better than
  `llama-mtmd-cli` for quality.

### Strategy: Image Preprocessing / Image Token Controls

Action:

- Confirmed the harness already applies EXIF orientation and writes prepared
  PNGs.
- Targeted prepared image sizes:
  - Browser screenshot: 3840x2088.
  - Chinese PDF pages: 2481x3508.
  - `sc-02`: 767x1081.
  - Rotated `upside-left`: 3024x4032.
- SGLang reference uses `image_mode=gundam`.
- llama.cpp exposes generic dynamic image-token controls but not an explicit
  SGLang-style `gundam`/`base` switch.
- Added `--image-min-tokens` and `--image-max-tokens` to both llama.cpp
  candidate paths.
- Ran a one-row smoke:
  - Candidate engine: `llamacpp-q4_k_m-image-tokens-smoke`
  - Case: `613256554-...`
  - Profile: `grounding`
  - `--image-min-tokens 1024`
  - `--image-max-tokens 2048`
  - `--max-tokens 512`

Summary:

- `SUMMARY-image-tokens-smoke.md`

Decision:

- The preprocessing axis is now testable, but no tuned image-token setting has
  been validated yet. The mismatch between SGLang `gundam` mode and llama.cpp's
  MTMD preprocessing remains a likely root-cause area.

### Current Overall Decision

Do not wait passively for a better GGUF. Higher-quality GGUFs and BF16 were
tested on the target set and did not fix the failures.

The next practical work should focus on llama.cpp's DeepSeek-OCR/Unlimited-OCR
runtime path:

1. Inspect DeepSeek-OCR MTMD preprocessing and chat-template construction in
   llama.cpp against Baidu/SGLang `gundam` mode.
2. Determine whether the community GGUF conversion maps Unlimited-OCR metadata
   to DeepSeek-OCR in a way that loses an Unlimited-OCR-specific behavior.
3. Add a small visual/debug mode that records image token counts and tiling
   behavior per input.
4. If the mismatch is confirmed, prefer a focused llama.cpp patch or a small
   custom C++ wrapper over waiting for another Q4/Q5/Q6 quantization.

## 2026-06-26 DeepSeek-OCR / SGLang Gundam Parity Implementation

Question:

- Can we implement the likely blocker around prompt template, media-token
  handling, and SGLang gundam image preprocessing parity?

Implementation:

- Added `uocr-harness inspect-preprocessing`.
- Added preprocessing records to result JSON under `preprocessing`.
- Added llama.cpp media-placement controls:
  - `auto`
  - `prefix-tight`
  - `prefix-newline`
  - `suffix-newline`
- Added SGLang media-placement controls:
  - `separate`
  - `prefix-tight`
  - `prefix-newline`
  - `suffix-newline`
- Added an opt-in llama.cpp DeepSeek-OCR preprocessing mode:
  - Harness flag: `--deepseek-ocr-mode gundam`
  - Runtime env: `LLAMA_DEEPSEEK_OCR_GUNDAM=1`
- Patched `thirdparty/llama.cpp` so original DeepSeek-OCR can:
  - select SGLang-style 640 local crop grids with min 2 / max 32 tiles;
  - append a 1024 padded global overview;
  - attach the view separator only to the global overview.
- Rebuilt:
  - `thirdparty/llama.cpp/build/bin/llama-mtmd-cli`
  - `thirdparty/llama.cpp/build/bin/llama-server`

Preprocessing inspection result:

- Wrote 26 rows to
  `unlimited-ocr-portable/results/inspection/preprocessing.jsonl`.
- Every current prepared case has a multi-column SGLang gundam crop grid.
- Native llama.cpp DeepSeek-OCR image tokens remain 273 for the 1024 global
  image.
- SGLang gundam adds local crop tokens:
  - `sc-02`: 3x4 crop grid, 1513 total SGLang image tokens.
  - Browser screenshot: 7x4 crop grid, 3113 total SGLang image tokens.
  - Chinese PDF pages: 4x6 crop grid, 2733 total SGLang image tokens.
- The experimental llama.cpp crop path is closer but still not exact because it
  emits independent per-tile newline embeddings. SGLang combines all local crop
  embeddings into one local grid and emits one newline per combined local row.

Focused smoke runs:

| Candidate | Case/Profile | Similarity | Repetition | Status |
|---|---|---:|---:|---|
| `llamacpp-q4_k_m-prompts` | `sc-02` / `document_parsing` | 0.011 | 0.773 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-prefix-tight` | `sc-02` / `document_parsing` | 0.237 | 0.530 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-rp105-prefix-tight` | `sc-02` / `document_parsing` | 0.458 | 0.588 | `candidate_repetition` |

Summaries:

- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-q4-prompts-sc02-document.md`

Decision:

- Yes, the parity work is implementable, and the first implementation improves
  a targeted smoke result materially.
- No, this implementation is not yet production quality. It still fails the
  repetition threshold and is not exact for any current dataset case because all
  current cases use multi-column crop grids.
- The next code-level fix was to implement true SGLang local-grid embedding
  composition inside the DeepSeek-OCR projector path: batch/encode local crop
  features, reorder rows across tile columns, drop extra per-tile newline
  embeddings, then append the 1024 global view and a single view separator. The
  following section records that implementation.

## 2026-06-26 Exact SGLang Local-Grid Composition

Question:

- Can the remaining SGLang local-grid blocker be implemented exactly in C/C++?

Implementation:

- Patched `thirdparty/llama.cpp/tools/mtmd/mtmd.cpp` so the DeepSeek-OCR
  gundam path keeps local tiles in one internal image chunk instead of emitting
  each tile as an independent chunk.
- Added DeepSeek-OCR gundam metadata to `mtmd_image_tokens`: grid width, grid
  height, tile side, and a composed-layout flag.
- Added a dedicated encode path that:
  - encodes each 640 local tile;
  - copies patch embeddings into one combined local grid;
  - emits one newline embedding per combined local row;
  - encodes the 1024 overview after the local grid;
  - appends the global view separator only through the overview image.
- Kept the mode opt-in through `LLAMA_DEEPSEEK_OCR_GUNDAM=1` and
  `uocr-harness run-llamacpp --deepseek-ocr-mode gundam`.
- Rebuilt both `llama-mtmd-cli` and `llama-server` from the patched tree.
- Updated `uocr-harness inspect-preprocessing` so active gundam totals now
  report composed SGLang-equivalent token counts while retaining independent
  per-tile counts as diagnostics.

Build verification:

```sh
make -C thirdparty/llama.cpp/build llama-mtmd-cli llama-server -j8
```

Harness verification:

```sh
uv run --project unlimited-ocr-portable uocr-harness inspect-preprocessing
uv run --project unlimited-ocr-portable -m compileall unlimited-ocr-portable/uocr_harness
```

Recorded preprocessing examples:

- Browser screenshot: SGLang total 3113 image tokens; patched llama.cpp gundam
  total 3113 image tokens; old independent-tile diagnostic total 3353.
- Chinese PDF pages: SGLang total 2733 image tokens; patched llama.cpp gundam
  total 2733 image tokens.
- `sc-02`: SGLang total 1513 image tokens; patched llama.cpp gundam total 1513
  image tokens.

Focused smoke results:

| Candidate | Case/Profile | Similarity | Repetition | Boxes | Status |
|---|---|---:|---:|---:|---|
| `llamacpp-q4_k_m-prompts` | `sc-02` / `document_parsing` | 0.011 | 0.773 | 0 / 9 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-prefix-tight` | `sc-02` / `document_parsing` | 0.237 | 0.530 | 9 / 9 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-rp105-prefix-tight` | `sc-02` / `document_parsing` | 0.458 | 0.588 | 9 / 9 | `candidate_repetition` |
| `llamacpp-q4_k_m-gundam-exact-prefix-tight` | `sc-02` / `document_parsing` | 0.998 | 0.191 | 9 / 9 | `pass` |
| `llamacpp-q4_k_m-gundam-exact-rp105-prefix-tight` | `sc-02` / `document_parsing` | 0.997 | 0.190 | 9 / 9 | `pass` |

Target-set results for `document_parsing` on five cases:

- `llamacpp-q4_k_m-gundam-exact-target-doc`:
  - 3 pass, 2 candidate repetition.
  - Average text similarity: 0.568.
  - Average reference bbox markers: 15.6.
  - Average candidate bbox markers: 16.4.
  - Remaining repetition failures: browser screenshot and rotated image.
- `llamacpp-q4_k_m-gundam-exact-rp105-target-doc`:
  - 2 pass, 3 candidate repetition.
  - Average text similarity: 0.428.
  - Average candidate bbox markers: 69.8.
  - Repeat penalty 1.05 worsened this target set by causing marker
    overproduction on `chinese-paper-page-0002`.

Full baseline-profile result:

- `llamacpp-q4_k_m-gundam-exact-full`:
  - 52 / 52 candidate result files.
  - 8 pass, 36 candidate empty, 5 candidate repetition, 1 bbox mismatch,
    1 malformed-marker row, 1 review row.
  - Comparable non-empty pairs: 16 / 52, up from 11 / 52 in the native Q4
    baseline.
  - Average text similarity across comparable pairs: 0.691, up from 0.550 in
    the native Q4 baseline.
  - Average candidate elapsed: 3170 ms.
  - Average candidate GPU-after-request snapshot: 2885 MB.

Updated summaries:

- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-full.md`

Decision:

- Yes, exact SGLang local-grid embedding composition is feasible and now
  implemented in C/C++.
- The local-grid blocker no longer explains the best current candidate
  failures.
- The full exact run is materially better than the native Q4 baseline but still
  not production-ready because most baseline-profile rows are empty. The
  remaining investigation should focus on SGLang custom no-repeat/logit
  processor behavior, decoding parity, prompt/profile details,
  orientation-sensitive cases, and broader validation once those are isolated.

## 2026-06-26 Custom Branch Parity Controls and Full Rerun

Question:

- Can the remaining SGLang parity gaps be closed by patching llama.cpp further
  on a local custom branch, and what is the best full-run candidate after those
  patches?

Local branch:

- `thirdparty/llama.cpp` branch: `uocr-deepseek-ocr-parity`.
- New local commit: `8fbbd5b mtmd-cli: add OCR sampling parity controls`.
- Existing grid commit: `3ebff83 mtmd: add Unlimited-OCR gundam grid parity`.
- Base upstream commit: `9d5d882 model : Add label for LFM2.5-230M (#25008)`.

Implementation:

- Added `common_sampler_sample_with_banned()` so llama.cpp callers can ban a
  small token set immediately before deterministic sampling.
- Implemented SGLang-style DeepSeek-OCR no-repeat ngram controls in
  `llama-mtmd-cli`.
- Recorded prompt-origin tokens for the no-repeat processor, including text
  chunk tokens and media sentinel spans.
- Added prefill-aware SWA, image-end newline suppression, and min-new-token
  diagnostic switches.
- Extended `uocr-harness run-llamacpp` with matching flags and result metadata.

SGLang findings:

- The repo `.venv` custom wheel provides
  `DeepseekOCRNoRepeatNGramLogitProcessor`.
- SGLang default no-repeat params are 30 / 90 with whitelist
  `[128821, 128822]`.
- The Unlimited-OCR processor strips prompt EOS in inference mode, despite
  constructing the internal sequence with `eos=True`.
- The effective SGLang OpenAI prompt is `<image>` immediately followed by the
  prompt text; roles and separators are empty.
- GGUF does not include `sliding_window_size=128`, so SWA128 has to stay an
  explicit candidate experiment.

Target experiments:

- Q4 forced EOS + SGLang no-repeat default: 8 pass / 20, 8 repetition,
  average similarity 0.464.
- Q4 forced EOS + no-repeat + SWA128: 9 pass / 20, 6 repetition,
  average similarity 0.502. This was the best Q4 target setting.
- Q4 forced EOS + no-repeat + no image-end newline: 8 pass / 20, 10 repetition,
  average similarity 0.476. Suppressing the newline was worse.
- Q4 no forced EOS + no-repeat + min-new-token 1: 7 pass / 20, 10 repetition,
  average similarity 0.417. It removed empties but made repetition worse.
- BF16 forced EOS + no-repeat default: 10 pass / 20, 7 repetition,
  average similarity 0.513.
- BF16 forced EOS + no-repeat + SWA128: 10 pass / 20, 7 repetition,
  average similarity 0.508.

Full reruns:

- Reran SGLang BF16 reference with all four profiles:
  `grounding,plain_text,ocr_boxes,document_parsing`.
  - 104 / 104 reference files.
  - SGLang log ended with graceful shutdown on 2026-06-26 at 22:20:58 local
    log time.
- Reran best Q4 candidate:
  `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full`.
  - 104 / 104 candidate files.
  - 56 pass, 17 repetition, 14 low similarity, 14 bbox mismatch, 3 review.
  - 0 empty outputs.
  - Average similarity 0.688.
  - Average candidate elapsed 3809 ms.
  - Average candidate GPU-after-request snapshot 1528 MB.
- Reran BF16 candidate as a quality ceiling:
  `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-full`.
  - 54 pass, 27 repetition, 9 low similarity, 6 bbox mismatch, 6 review,
    2 malformed-marker rows.
  - Average similarity 0.649.
  - Average candidate elapsed 9743 ms.

Updated summaries:

- `SUMMARY.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-noimgend-target.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-minnew1-target.md`

Decision:

- Parity is not achieved.
- The wrapper/MTMD-level gaps that were feasible to implement have now been
  implemented or tested as diagnostics.
- Q4 is the best current full-run candidate; BF16 did not improve the full
  matrix.
- Stable unpatched llama-server is not sufficient for this model path. Use the
  local patched llama.cpp branch as the current base reference.
- Production packaging should not proceed as a stable release yet. The next
  meaningful work is deeper llama.cpp runtime/model parity with SGLang, or
  upstreaming/replacing these focused patches in a small native wrapper.

## 2026-06-26 Native Parity Runner and Artifact Instrumentation

Question:

- Can we keep extending a custom llama.cpp branch through a cloned
  `llama-mtmd-cli` / `llama-server` path, and is there still C++ work that can
  improve the candidate?

Local branch:

- `thirdparty/llama.cpp` branch: `uocr-deepseek-ocr-parity`.
- New local commit: `48f8954 mtmd-cli: add Unlimited-OCR parity artifact runner`.
- Prior local commits retained:
  - `8fbbd5b mtmd-cli: add OCR sampling parity controls`.
  - `3ebff83 mtmd: add Unlimited-OCR gundam grid parity`.

Implementation:

- Added a named native target, `llama-uocr-parity`, built from the patched MTMD
  CLI path.
- Added opt-in artifact dumping through `LLAMA_UOCR_PARITY_DUMP` and
  `LLAMA_UOCR_PARITY_TOPK`.
- Native artifacts include formatted prompt text, text/media token counts, image
  chunk summaries, image embedding numeric summaries, prefill top logits, and
  per-token generation top-k data.
- Extended `uocr-harness run-llamacpp` with `--debug-artifacts` and
  `--debug-top-k`.
- Extended `uocr-harness run-sglang` with chat-logprob artifact capture through
  `/v1/chat/completions`.
- Added `uocr-harness compare-artifacts` for SGLang/candidate artifact
  comparison.

Validation:

- Rebuilt `llama-mtmd-cli`, `llama-uocr-parity`, and `llama-server`.
- Recompiled the Python harness with `uv run --project unlimited-ocr-portable -m
  compileall unlimited-ocr-portable/uocr_harness`.
- Captured SGLang and llama.cpp artifacts for `sc-02-45a8efac` /
  `document_parsing`.
- Baseline artifact summary: `SUMMARY-parity-artifacts-smoke.md`.
- No-image-end diagnostic artifact summary:
  `SUMMARY-parity-artifacts-noimgend-smoke.md`.

Artifact finding:

- SGLang's first API-visible token is `<|det|>`.
- The patched llama.cpp candidate emits raw newline token `201` first, then
  `<|det|>`.
- Later runtime-token inspection refined the image-end finding: removing the
  DeepSeek-OCR image-end newline removes the prefill newline, but the earlier
  no-image-end artifact still had forced EOS.
- The earlier target-set run showed `--deepseek-ocr-no-image-end` was worse:
  8 pass / 20, 10 repetition, average similarity 0.476, versus the best Q4
  target setting at 9 pass / 20, 6 repetition, average similarity 0.502.

Decision:

- Yes, a custom branch can still carry focused native C++ instrumentation and
  wrapper-style experiments. The current implementation makes that practical
  without forking a separate product server yet.
- No, the latest evidence does not justify more prompt-boundary or image-end
  switches as a default. Those are now tested and mostly exhausted.
- The remaining feasible C++ work is deeper parity instrumentation: hidden-state
  / logits comparison, attention/SWA behavior, tokenizer/template internals, or
  model-specific runtime differences between llama.cpp and Baidu's custom
  SGLang path.
- Until exact parity improves, production should stay on a pinned patched
  llama.cpp branch for experiments, not stable unpatched llama-server.

## 2026-06-27 Runtime Token and Native Logprob Parity

Objective:

- Implement the next deeper parity step after native artifact dumping:
  tokenizer/template internals, native SGLang input logprobs, and exact
  prompt/media prefill comparison.

Implementation:

- Added `uocr-harness inspect-sglang-processor`.
  - Uses the custom SGLang wheel's `UnlimitedOCRHFProcessor` from `.venv`.
  - Writes `results/artifacts/reference/sglang-processor/<case>/<profile>.processor.json`.
  - Summarizes processor input IDs, image-token runs, spatial crop, tensor
    shapes/statistics, tokenizer special-token IDs, and SGLang model metadata.
- Added `uocr-harness compare-runtime-parity`.
  - Compares SGLang processor artifacts with llama.cpp native artifacts.
  - Reports media-token equality, non-image token equality, and prefill delta.
- Added SGLang native `/generate` debug artifacts to `run-sglang`.
  - `--debug-native-artifacts` requests `return_logprob`,
    `logprob_start_len=0`, and `top_logprobs_num`.
  - `--debug-return-hidden-states` can request hidden states when the server is
    started with `--enable-return-hidden-states`.
  - Artifacts are written under `results/artifacts/reference/sglang-native/`.
- Extended `compare-artifacts` to read native SGLang artifacts and compare them
  against llama.cpp native `LLAMA_UOCR_PARITY_DUMP` artifacts.

Validation:

- Harness compile passed with:
  `uv run --project unlimited-ocr-portable -m compileall unlimited-ocr-portable/uocr_harness`.
- SGLang processor artifact for `sc-02-45a8efac` / `document_parsing`:
  - Total input tokens: 1517.
  - Image tokens: 1513.
  - Non-image tokens: `[0, 34030, 76466, 16]`.
  - Spatial crop: `[[[3, 4]]]`.
- Runtime parity comparisons:
  - Forced EOS candidate: `[0, 201, 34030, 76466, 16, 1]`, prefill delta 2.
  - No forced EOS candidate: `[0, 201, 34030, 76466, 16]`, prefill delta 1.
  - No image-end with forced EOS: `[0, 34030, 76466, 16, 1]`, prefill delta 1.
  - No forced EOS plus no image-end: `[0, 34030, 76466, 16]`, prefill delta 0,
    `runtime_sequence_match`.
- Native SGLang `/generate` artifact run completed with a one-token request:
  - Prompt tokens: 1517.
  - `input_token_logprobs`: 1517 rows.
  - First output token: `<|det|>` / token `128818`.
- Isolated hidden-state validation completed under `/tmp/uocr-hidden-results`:
  - Prompt tokens: 1517.
  - Completion tokens: 1.
  - Summarized hidden-state shape: `[1, 1517, 1280]`.
- Matching llama.cpp one-token exact-prefill artifact:
  - Prefill tokens: 1517.
  - First output token: `<|det|>` / token `128818`.
  - First-output top-k overlap with SGLang native: 1.000.
- Exact-prefill 20-row target run:
  `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target`.
  - 10 pass, 4 repetition, 6 low similarity.
  - Average similarity 0.512 after restoring the full SGLang `sc-02` /
    `document_parsing` reference row.
  - Non-empty outputs: 20 / 20.

Decision:

- The earlier no-image-end conclusion needed refinement. No-image-end does
  remove the prefill newline; the remaining extra token in that artifact was
  forced EOS.
- Exact processor prefill parity is now implemented and validated for the smoke
  case without a new C++ patch, by combining no forced EOS with
  `--deepseek-ocr-no-image-end`.
- First-token native SGLang logits/top-k align with llama.cpp under exact
  prefill.
- Multi-token OCR quality still does not match SGLang. Exact prefill slightly
  improves the target set, but the remaining blocker is still later-token
  runtime divergence: attention/KV behavior, hidden-state drift, numeric/runtime
  differences, or model implementation differences after the first generated
  token.

## 2026-06-27 Generation-Step Runtime Parity

Objective:

- Continue the deeper runtime parity work by comparing generated token IDs and
  top-k rankings step by step after exact processor prefill parity.

Implementation:

- Added `uocr-harness compare-generation-artifacts`.
  - Reads SGLang native `/generate` artifacts from `run-sglang
    --debug-native-artifacts`.
  - Reads llama.cpp `LLAMA_UOCR_PARITY_DUMP` artifacts from
    `llama-uocr-parity`.
  - Writes pair-level metrics plus per-step CSV rows.
  - Reports matching prefix length, first divergence step, top-k overlap, token
    ranks, and score margins at the first divergence.

Validation:

- Recompiled the harness with:
  `uv run --project unlimited-ocr-portable -m compileall
  unlimited-ocr-portable/uocr_harness`.
- Captured isolated 64-token artifacts under `/tmp/uocr-step-results` for
  `sc-02-45a8efac` / `document_parsing`.
- Q4 exact-prefill candidate:
  - Summary: `SUMMARY-generation-steps-noimgend-noeos-64tok.md`.
  - Matching prefix: 3 generated tokens, `<|det|>`, `header`, ` [`.
  - First divergence: step 3, first bbox coordinate.
  - SGLang token: `6207` / `91`.
  - Q4 token: `6152` / `92`.
  - SGLang ranks `91` first and `92` second with a 0.25 logprob margin.
  - Q4 ranks `92` first and `91` third with a 0.972 raw-logit margin.
- Q4 without the prefill-aware SWA experiment:
  - Summary: `SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok.md`.
  - Same first divergence as Q4 with SWA, so the SWA flag is not the first
    cause.
- Q5_K_M, Q6_K, and BF16 exact-prefill checks:
  - Summaries:
    - `SUMMARY-generation-steps-q5-noimgend-noeos-64tok.md`
    - `SUMMARY-generation-steps-q6-noimgend-noeos-64tok.md`
    - `SUMMARY-generation-steps-bf16-noimgend-noeos-64tok.md`
  - All diverged earlier at generation step 1 by ranking `aside` over SGLang's
    `header`.
- Full exact-prefill/no-image-end Q4 matrix:
  - Summary: `SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`.
  - 104 reference rows and 104 candidate rows.
  - Status counts: 49 pass, 27 repetition, 10 bbox mismatch, 7 review, 6 low
    similarity, 5 empty.
  - Average similarity: 0.671.

Decision:

- Exact prefill and first-token top-k parity are achieved for the smoke case,
  but this is not enough for production output parity.
- Do not promote the exact-prefill/no-image-end setting to the default full-run
  candidate; it regresses from the current best 56 / 104 passes to 49 / 104.
- Higher GGUF precision does not solve the first stepwise divergence in this
  diagnostic. The remaining blocker is now deeper runtime/model numeric parity
  after generation begins, not local-grid composition, tokenizer/template
  layout, image-boundary tokens, first-token logits, or the tested SWA/no-repeat
  switches.
