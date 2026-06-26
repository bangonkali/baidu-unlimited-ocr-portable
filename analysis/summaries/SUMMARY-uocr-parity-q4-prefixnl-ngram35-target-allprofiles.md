# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:09:42+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-prefixnl-ngram35-target-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-prefixnl-ngram35-target-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_empty`: 5
- `candidate_malformed_markers`: 1
- `candidate_repetition`: 10
- `pass`: 3

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
- `SUMMARY-uocr-parity-bf16-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-ngram-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram4-target-allprofiles.md`
- `SUMMARY-uocr-placement-auto.md`
- `SUMMARY-uocr-placement-prefix-newline.md`
- `SUMMARY-uocr-placement-prefix-tight.md`
- `SUMMARY-uocr-placement-suffix-newline.md`

## Aggregate Metrics

- Reference result files: 20 / 20
- Candidate result files: 20 / 20
- Comparable pairs: 15 / 20
- Candidate non-empty outputs: 15 / 20
- Candidate empty outputs: 5 / 20
- Candidate high-repetition rows: 10 / 20
- Candidate malformed-marker rows: 6 / 20
- Rows with >30% bbox-count delta: 14 / 20
- Average text similarity: 0.425
- Average reference elapsed: 9808 ms
- Average candidate elapsed: 5687 ms
- Average reference GPU after request: 31623 MB
- Average candidate GPU after request: 1723 MB
- Average reference bbox markers: 82.250
- Average candidate bbox markers: 56.250

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-prefixnl-ngram35-target-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.093 | 48 | 306 | 10086 | 0.854 | 13375 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 2075 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.001 | 1 | 73 | 35660 | 0.978 | 13230 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.019 | 1 | 41 | 608 | 0.482 | 3349 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | grounding | 0.933 | 18 | 17 | 2732 | 0.061 | 3023 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0001-2200885e | plain_text | 0.059 | 18 | 73 | 22403 | 0.886 | 12801 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 1949 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | plain_text | n/a | 305 | 0 | 0 | 0.000 | 2008 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.116 | 327 | 301 | 13005 | 0.921 | 12949 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | document_parsing | 0.503 | 45 | 79 | 3339 | 0.480 | 4605 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
