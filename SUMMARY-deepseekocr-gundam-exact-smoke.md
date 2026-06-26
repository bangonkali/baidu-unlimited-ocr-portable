# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T11:38:33+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-gundam-exact-prefix-tight`
- Metrics CSV: `results/compare/SUMMARY-deepseekocr-gundam-exact-smoke.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `pass`: 1

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-prompts-sc02-document.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`

## Aggregate Metrics

- Reference result files: 1 / 1
- Candidate result files: 1 / 1
- Comparable pairs: 1 / 1
- Candidate non-empty outputs: 1 / 1
- Candidate empty outputs: 0 / 1
- Candidate high-repetition rows: 0 / 1
- Candidate malformed-marker rows: 0 / 1
- Rows with >30% bbox-count delta: 0 / 1
- Average text similarity: 0.998
- Average reference elapsed: 1881 ms
- Average candidate elapsed: 5600 ms
- Average reference GPU after request: 31574 MB
- Average candidate GPU after request: 2901 MB
- Average reference bbox markers: 9.000
- Average candidate bbox markers: 9.000

## Quality Finding

- Candidate outputs passed the automated checks in this harness.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

No review items.

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
