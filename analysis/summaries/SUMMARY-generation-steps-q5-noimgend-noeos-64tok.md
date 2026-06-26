# Unlimited-OCR Generation Step Artifact Summary

Generated: 2026-06-26T16:33:51+00:00

## Engines

- Reference: `sglang-native`
- Candidate: `llamacpp-q5_k_m-uocr-parity-debug-noimgend-noeos-64tok`
- Metrics CSV: `/tmp/uocr-step-results/compare/SUMMARY-generation-steps-q5-noimgend-noeos-64tok.csv`
- Steps CSV: `/tmp/uocr-step-results/compare/SUMMARY-generation-steps-q5-noimgend-noeos-64tok-steps.csv`

## Status Counts

- `generation_diverged_after_prefix`: 1

## Aggregate Findings

- Rows compared: 1
- Average matching prefix tokens: 1.000
- Earliest divergence step: 1
- Average top-k overlap: 0.059

## Review Queue

| Status | Case | Profile | Ref Steps | Candidate Steps | Matching Prefix | First Divergence | Ref Token | Cand Token | Cand Rank In Ref Top | Ref Rank In Cand Top | Ref Margin | Cand Margin | Avg Top Overlap |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| generation_diverged_after_prefix | sc-02-45a8efac | document_parsing | 64 | 64 | 1 | 1 | 16771:header | 121695:aside | 2 | 2 | 1.125 | 0.122288 | 0.059 |

## Notes

- This compares generated token IDs and top-k token ID overlap step by step.
- Steps after the first token mismatch are no longer conditioned on the same prefix, so the first divergence is the primary debugging signal.
- Use native SGLang `/generate` artifacts for token IDs and input/output logprobs.
