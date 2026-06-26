# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T12:59:53+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-ngram-target-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-ngram-target-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_empty`: 7
- `candidate_repetition`: 4
- `low_similarity`: 1
- `pass`: 7

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

## Aggregate Metrics

- Reference result files: 20 / 20
- Candidate result files: 20 / 20
- Comparable pairs: 13 / 20
- Candidate non-empty outputs: 13 / 20
- Candidate empty outputs: 7 / 20
- Candidate high-repetition rows: 4 / 20
- Candidate malformed-marker rows: 4 / 20
- Rows with >30% bbox-count delta: 11 / 20
- Average text similarity: 0.591
- Average reference elapsed: 9808 ms
- Average candidate elapsed: 4533 ms
- Average reference GPU after request: 31623 MB
- Average candidate GPU after request: 1741 MB
- Average reference bbox markers: 82.250
- Average candidate bbox markers: 65.000

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-ngram-target-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 2363 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 2165 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.012 | 1 | 71 | 7079 | 0.635 | 12283 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.017 | 1 | 49 | 1836 | 0.103 | 3736 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 1925 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | plain_text | n/a | 305 | 0 | 0 | 0.000 | 1959 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.222 | 327 | 318 | 11017 | 0.404 | 12798 | dataset/chinese-paper.pdf |
| candidate_empty | sc-02-45a8efac | grounding | n/a | 9 | 0 | 0 | 0.000 | 1590 | dataset/sc-02.png |
| bbox_count_mismatch | sc-02-45a8efac | plain_text | 0.918 | 9 | 5 | 2828 | 0.165 | 2582 | dataset/sc-02.png |
| candidate_empty | upside-left-9e645a2a | grounding | n/a | 402 | 0 | 0 | 0.000 | 1871 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
