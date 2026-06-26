# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T10:36:50+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-prompts`
- Metrics CSV: `results/compare/SUMMARY-q4-prompts-sc02-document.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_repetition`: 1

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
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
- Candidate high-repetition rows: 1 / 1
- Candidate malformed-marker rows: 0 / 1
- Rows with >30% bbox-count delta: 0 / 1
- Average text similarity: 0.011
- Average reference elapsed: 1881 ms
- Average candidate elapsed: 11321 ms
- Average reference GPU after request: 31574 MB
- Average candidate GPU after request: 2057 MB
- Average reference bbox markers: 9.000
- Average candidate bbox markers: 8.000

## Quality Finding

- `llamacpp-q4_k_m-prompts` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | sc-02-45a8efac | document_parsing | 0.011 | 9 | 8 | 16196 | 0.773 | 11321 | dataset/sc-02.png |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
