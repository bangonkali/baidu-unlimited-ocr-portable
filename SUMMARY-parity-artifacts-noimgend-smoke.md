# Unlimited-OCR Parity Artifact Summary

Generated: 2026-06-26T15:41:20+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-debug-noimgend-smoke`
- Metrics CSV: `results/compare/SUMMARY-parity-artifacts-noimgend-smoke.csv`

## Status Counts

- `candidate_leading_whitespace_token`: 1

## Finding

- The candidate emits a raw leading whitespace token after prefill before the first visible OCR token.

## Review Queue

| Status | Case | Profile | Ref first | Cand first | Cand raw | Prefill top overlap | Output top overlap | Ref tokens | Cand tokens |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| candidate_leading_whitespace_token | sc-02-45a8efac | document_parsing | <|det|> | 201 -> <|det|> | 201 | 0.125 | 0.125 | 128 | 128 |

## Notes

- SGLang artifacts use `/v1/chat/completions` logprob metadata from the same chat path as the reference run.
- llama.cpp artifacts use `LLAMA_UOCR_PARITY_DUMP` from the patched native MTMD path.
- SGLang chat logprobs expose token text/top logprobs but not internal image embedding tensors or token IDs.
