# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T12:52:54+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-bf16-uocr-parity-grid-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-bf16-grid-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 6
- `candidate_empty`: 35
- `candidate_repetition`: 26
- `low_similarity`: 1
- `pass`: 33
- `review`: 3

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
- `SUMMARY-uocr-parity-q4-grid-allprofiles.md`

## Aggregate Metrics

- Reference result files: 104 / 104
- Candidate result files: 104 / 104
- Comparable pairs: 68 / 104
- Candidate non-empty outputs: 69 / 104
- Candidate empty outputs: 35 / 104
- Candidate high-repetition rows: 26 / 104
- Candidate malformed-marker rows: 12 / 104
- Rows with >30% bbox-count delta: 58 / 104
- Average text similarity: 0.622
- Average reference elapsed: 6297 ms
- Average candidate elapsed: 8905 ms
- Average reference GPU after request: 31735 MB
- Average candidate GPU after request: 1734 MB
- Average reference bbox markers: 69.365
- Average candidate bbox markers: 57.971

## Quality Finding

- `llamacpp-bf16-uocr-parity-grid-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.017 | 48 | 9 | 28617 | 0.975 | 32590 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.999 | 1 | 0 | 8264 | 0.996 | 26765 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 1.000 | 1 | 1 | 8259 | 0.996 | 25408 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.013 | 1 | 1 | 8259 | 0.996 | 25176 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | 0.001 | 1 | 1 | 23125 | 0.987 | 25181 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | 0.020 | 307 | 0 | 8264 | 0.996 | 25165 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | ocr_boxes | n/a | 1 | 0 | 0 | 0.000 | 2855 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | document_parsing | 0.009 | 1 | 1 | 8224 | 0.997 | 25177 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | chinese-paper-page-0001-2200885e | plain_text | 0.000 | 18 | 0 | 19111 | 0.999 | 25097 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 2638 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
