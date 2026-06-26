# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T14:29:07+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full`
- Metrics CSV: `results/compare/metrics.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 14
- `candidate_repetition`: 17
- `low_similarity`: 14
- `pass`: 56
- `review`: 3

## Related Strategy Summaries

- `SUMMARY-parity-artifacts-smoke.md`
- `SUMMARY-parity-artifacts-noimgend-smoke.md`
- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-exact-full.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-target-doc.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-generation-steps-bf16-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok.md`
- `SUMMARY-generation-steps-q5-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q6-noimgend-noeos-64tok.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-parity-artifacts-output-embeddings-onetok.md`
- `SUMMARY-q4-prompts-sc02-document.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-bf16-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-swa128-full.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-noimgend-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram16-target.md`
- `SUMMARY-uocr-parity-q4-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-ngram-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-minnew1-target.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-prefixnl-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram4-target-allprofiles.md`
- `SUMMARY-uocr-placement-auto.md`
- `SUMMARY-uocr-placement-prefix-newline.md`
- `SUMMARY-uocr-placement-prefix-tight.md`
- `SUMMARY-uocr-placement-suffix-newline.md`

## Aggregate Metrics

- Reference result files: 104 / 104
- Candidate result files: 104 / 104
- Comparable pairs: 99 / 104
- Candidate non-empty outputs: 104 / 104
- Candidate empty outputs: 0 / 104
- Candidate high-repetition rows: 17 / 104
- Candidate malformed-marker rows: 10 / 104
- Rows with >30% bbox-count delta: 44 / 104
- Average text similarity: 0.688
- Average reference elapsed: 5797 ms
- Average candidate elapsed: 3809 ms
- Average reference GPU after request: 31654 MB
- Average candidate GPU after request: 1528 MB
- Average reference bbox markers: 70.135
- Average candidate bbox markers: 48.154

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Latest Parity Artifact Finding

- Local branch `uocr-deepseek-ocr-parity` now includes commit `7b0ec28` and the
  `llama-uocr-parity` native debug runner.
- On `sc-02-45a8efac` / `document_parsing`, SGLang's first API-visible token is
  `<|det|>`. llama.cpp emits raw newline token `201` first, then the same
  visible `<|det|>` token.
- Later runtime inspection refined the no-image-end result: no-image-end removes
  the prefill newline, but the earlier artifact still had forced EOS. Exact
  SGLang prefill parity requires both no forced EOS and
  `--deepseek-ocr-no-image-end`.
- Next useful C++ work needs deeper runtime/model parity instrumentation rather
  than more prompt or image-boundary switches.

## Runtime Parity Finding

- Added SGLang processor/template inspection and native `/generate` logprob
  artifacts in the portable harness.
- SGLang processor input for `sc-02-45a8efac` / `document_parsing` is exactly
  1517 tokens: 1513 image tokens plus `[0, 34030, 76466, 16]`.
- The forced-EOS debug candidate was two tokens longer:
  `[0, 201, 34030, 76466, 16, 1]`.
- The no-forced-EOS candidate was one token longer:
  `[0, 201, 34030, 76466, 16]`.
- Combining no forced EOS with `--deepseek-ocr-no-image-end` gives exact
  SGLang processor prefill parity: 1517 / 1517 tokens.
- One-token native SGLang `/generate` and llama.cpp exact-prefill artifacts
  align on first token `<|det|>` with first-output top-k overlap 1.000.
- Hidden-state return plumbing was validated in an isolated `/tmp` results run:
  SGLang returned summarized hidden states with shape `[1, 1517, 1280]`.
- llama.cpp output-embedding summaries were added to the native artifact path
  via `LLAMA_UOCR_PARITY_OUTPUT_EMBEDDINGS=1` /
  `--debug-output-embeddings`. The one-token smoke captured a 1280-wide
  prefill-last output embedding and one generated-token embedding, while the
  paired SGLang native artifact exposed hidden-state shape `[1, 1517, 1280]`.
  See `SUMMARY-parity-artifacts-output-embeddings-onetok.md`.
- Exact prefill is not enough for output parity: the 20-row target run
  `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-target` reached 10 pass / 20,
  4 repetition rows, 6 low-similarity rows, and average similarity 0.512.
  That slightly improves the target-set average over the prior Q4 target
  setting at 0.502, but it is still not production-ready and has not replaced
  the current best 104-row full-run baseline.
- The full 104-row exact-prefill/no-image-end run regressed relative to the
  current best baseline: 49 pass / 104, 5 empty rows, 27 repetition rows, and
  average similarity 0.671 in `SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`.
- The full 104-row exact-prefill/no-image-end/SWA128 run ties the current
  56-pass baseline and improves average similarity to 0.717, but introduces
  5 empty rows and 17 low-similarity rows. It is a useful alternate candidate
  for follow-up, not production parity.

## Candidate-Best Client Demo

- Added `candidate-best-client/`, a Gradio demo that calls the patched native
  `llama-uocr-parity` binary as a subprocess and streams generated stdout.
- Default demo profile:
  `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full`
  because it is the best zero-empty full-run candidate.
- Experimental demo profile:
  `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full` because it improves
  average similarity but produced 5 empty rows in the 104-row matrix.
- WSL2 validation on 2026-06-27:
  - compileall passed for the demo app and helper package.
  - default profile smoke on `dataset/sc-02.png` with 64 tokens exited 0 and
    produced visible `<|det|>` output in 2050 ms.
  - experimental profile smoke on the same image exited 0 and produced visible
    `<|det|>` output in 2438 ms.
  - PDF/parser smoke rendered 6 pages from `dataset/chinese-paper.pdf`, parsed
    1 marker box, and produced a preview image.
  - Gradio launched at `http://127.0.0.1:7861` and returned the expected app
    title/profile configuration.

## Generation-Step Parity Finding

- Added `compare-generation-artifacts`, which compares native SGLang
  `/generate` output token IDs/top-k against llama.cpp
  `LLAMA_UOCR_PARITY_DUMP` generation steps.
- On `sc-02-45a8efac` / `document_parsing`, exact-prefill Q4 matches SGLang
  for the first three generated tokens: `<|det|>`, `header`, and ` [`.
- The first Q4 divergence is generation step 3, the first bbox coordinate:
  SGLang selects token `6207` / `91`, while Q4 selects token `6152` / `92`.
  Both candidates are present in both top-k lists; SGLang ranks `91` first
  with a 0.25 logprob margin over `92`, while Q4 ranks `92` first with a
  0.972 raw-logit margin over `91`.
- Disabling the prefill-aware SWA experiment does not change that Q4 first
  divergence.
- Higher-weight GGUFs do not solve the stepwise issue. Q5_K_M, Q6_K, and BF16
  diverge earlier at step 1 by ranking `aside` over SGLang's `header`.
- This makes the remaining blocker later-token runtime/model numeric parity,
  not prompt template, local-grid composition, image-boundary tokens, first
  output-token logits, or the tested no-repeat/SWA switches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 22:20:58] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.056 | 48 | 413 | 4336 | 0.794 | 11808 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.003 | 1 | 91 | 34206 | 0.953 | 11609 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.013 | 1 | 78 | 6968 | 0.974 | 10691 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.013 | 1 | 1 | 8256 | 0.993 | 11678 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | 0.008 | 1 | 415 | 4268 | 0.840 | 11546 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | 0.259 | 307 | 43 | 632 | 0.443 | 3010 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | ocr_boxes | 0.014 | 1 | 315 | 12699 | 0.917 | 11610 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | document_parsing | 0.066 | 405 | 74 | 1107 | 0.641 | 3937 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.168 | 305 | 45 | 1732 | 0.000 | 3299 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.126 | 272 | 45 | 1732 | 0.000 | 3241 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
