# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T11:42:07+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-gundam-exact-rp105-target-doc`
- Metrics CSV: `results/compare/SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_repetition`: 3
- `pass`: 2

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-target-doc.md`
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

- Reference result files: 5 / 5
- Candidate result files: 5 / 5
- Comparable pairs: 5 / 5
- Candidate non-empty outputs: 5 / 5
- Candidate empty outputs: 0 / 5
- Candidate high-repetition rows: 3 / 5
- Candidate malformed-marker rows: 1 / 5
- Rows with >30% bbox-count delta: 2 / 5
- Average text similarity: 0.428
- Average reference elapsed: 8789 ms
- Average candidate elapsed: 10045 ms
- Average reference GPU after request: 31574 MB
- Average candidate GPU after request: 2893 MB
- Average reference bbox markers: 15.600
- Average candidate bbox markers: 69.800

## Quality Finding

- `llamacpp-q4_k_m-gundam-exact-rp105-target-doc` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.007 | 1 | 1 | 8218 | 0.998 | 14922 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | document_parsing | 0.193 | 45 | 314 | 10378 | 0.807 | 15084 | dataset/chinese-paper.pdf |
| candidate_repetition | upside-left-9e645a2a | document_parsing | 0.006 | 5 | 7 | 10223 | 0.982 | 14179 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
