# Unlimited-OCR Generation Step Artifact Summary

Generated: 2026-06-26T16:34:09+00:00

## Engines

- Reference: `sglang-native`
- Candidate: `llamacpp-q4_k_m-uocr-parity-debug-noimgend-noeos-noswa-64tok`
- Metrics CSV: `/tmp/uocr-step-results/compare/SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok.csv`
- Steps CSV: `/tmp/uocr-step-results/compare/SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok-steps.csv`

## Status Counts

- `generation_diverged_after_prefix`: 1

## Aggregate Findings

- Rows compared: 1
- Average matching prefix tokens: 3.000
- Earliest divergence step: 3
- Average top-k overlap: 0.807

## Review Queue

| Status | Case | Profile | Ref Steps | Candidate Steps | Matching Prefix | First Divergence | Ref Token | Cand Token | Cand Rank In Ref Top | Ref Rank In Cand Top | Ref Margin | Cand Margin | Avg Top Overlap |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| generation_diverged_after_prefix | sc-02-45a8efac | document_parsing | 64 | 64 | 3 | 3 | 6207:91 | 6152:92 | 2 | 3 | 0.25 | 0.972054 | 0.807 |

## Notes

- This compares generated token IDs and top-k token ID overlap step by step.
- Steps after the first token mismatch are no longer conditioned on the same prefix, so the first divergence is the primary debugging signal.
- Use native SGLang `/generate` artifacts for token IDs and input/output logprobs.
