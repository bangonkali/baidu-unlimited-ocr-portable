# Runtime Parity Summary

Generated: 2026-06-26T16:07:13+00:00

## Engines

- Reference artifact engine: `sglang-processor`
- Candidate artifact engine: `llamacpp-q4_k_m-uocr-parity-debug-noimgend-smoke`
- Metrics CSV: `results/compare/SUMMARY-runtime-parity-noimgend-smoke.csv`

## Status Counts

- `candidate_extra_boundary_tokens`: 1

## Aggregate Findings

- Rows compared: 1
- Image/media token count matches: 1 / 1
- Exact non-image text token matches: 0 / 1
- Matches after stripping candidate newline/EOS boundary tokens: 1 / 1
- Average candidate prefill delta vs SGLang processor input length: 1.000

## Interpretation

- `runtime_sequence_match` means llama.cpp's text chunks, media-token count, and prefill length match the SGLang processor artifact.
- `candidate_extra_boundary_tokens` means image tokens match and the prompt text matches only after removing candidate-only newline/EOS boundary tokens.
- This check does not run generation; it isolates tokenizer/template/media-token parity before logits and decoding differences.

## Review Rows

| Status | Case | Profile | Prefill Delta | Ref Text Tokens | Candidate Text Tokens |
|---|---|---|---:|---|---|
| candidate_extra_boundary_tokens | sc-02-45a8efac | document_parsing | 1 | `[0, 34030, 76466, 16]` | `[0, 34030, 76466, 16, 1]` |
