# Unlimited-OCR Parity Artifact Summary

Generated: 2026-06-26T16:22:53+00:00

## Engines

- Reference: `sglang-native`
- Candidate: `llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-onetok`
- Metrics CSV: `results/compare/SUMMARY-parity-artifacts-native-onetok.csv`

## Status Counts

- `api_visible_tokens_aligned`: 1

## Finding

- API-visible output tokens align for this set; move deeper to image embeddings, attention/SWA, or hidden-state instrumentation.

## Review Queue

No review items.

## Notes

- SGLang artifacts use `/generate` logprob metadata.
- llama.cpp artifacts use `LLAMA_UOCR_PARITY_DUMP` from the patched native MTMD path.
- Native SGLang `/generate` artifacts can include input logprobs; OpenAI chat artifacts only cover output logprobs.
