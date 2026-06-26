# Unlimited-OCR Parity Artifact Summary

Generated: 2026-06-26T17:06:31+00:00

## Engines

- Reference: `sglang-native`
- Candidate: `llamacpp-q4_k_m-uocr-parity-debug-output-embeddings-onetok`
- Metrics CSV: `/tmp/uocr-hidden-results/compare/SUMMARY-parity-artifacts-output-embeddings-onetok.csv`

## Status Counts

- `api_visible_tokens_aligned`: 1

## Finding

- API-visible output tokens align and hidden/output-embedding summaries are present for this set; remaining drift is beyond the first-token visible/logit/embedding smoke.

## Review Queue

No review items.

## Notes

- SGLang artifacts use `/generate` logprob metadata.
- llama.cpp artifacts use `LLAMA_UOCR_PARITY_DUMP` from the patched native MTMD path.
- Native SGLang `/generate` artifacts can include input logprobs; OpenAI chat artifacts only cover output logprobs.
