# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T12:35:44+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-grid-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-grid-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 4
- `candidate_empty`: 42
- `candidate_malformed_markers`: 2
- `candidate_repetition`: 25
- `low_similarity`: 1
- `pass`: 28
- `review`: 2

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-exact-full.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md`
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

- Reference result files: 104 / 104
- Candidate result files: 104 / 104
- Comparable pairs: 61 / 104
- Candidate non-empty outputs: 62 / 104
- Candidate empty outputs: 42 / 104
- Candidate high-repetition rows: 25 / 104
- Candidate malformed-marker rows: 14 / 104
- Rows with >30% bbox-count delta: 67 / 104
- Average text similarity: 0.607
- Average reference elapsed: 6297 ms
- Average candidate elapsed: 4847 ms
- Average reference GPU after request: 31735 MB
- Average candidate GPU after request: 2090 MB
- Average reference bbox markers: 69.365
- Average candidate bbox markers: 54.596

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-grid-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 2423 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 2283 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.997 | 1 | 1 | 8285 | 0.993 | 13433 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.007 | 1 | 1 | 8218 | 0.998 | 13343 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | n/a | 1 | 0 | 0 | 0.000 | 2092 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | n/a | 307 | 0 | 0 | 0.000 | 2211 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | ocr_boxes | n/a | 1 | 0 | 0 | 0.000 | 2188 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | document_parsing | 0.002 | 1 | 1 | 21625 | 0.993 | 13222 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 2000 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | plain_text | n/a | 305 | 0 | 0 | 0.000 | 1974 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
