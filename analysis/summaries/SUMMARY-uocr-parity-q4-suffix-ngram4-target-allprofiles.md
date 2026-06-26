# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:03:26+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-suffix-ngram4-target-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-suffix-ngram4-target-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_malformed_markers`: 3
- `candidate_repetition`: 2
- `low_similarity`: 15

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
- `SUMMARY-uocr-placement-auto.md`
- `SUMMARY-uocr-placement-prefix-newline.md`
- `SUMMARY-uocr-placement-prefix-tight.md`
- `SUMMARY-uocr-placement-suffix-newline.md`

## Aggregate Metrics

- Reference result files: 20 / 20
- Candidate result files: 20 / 20
- Comparable pairs: 20 / 20
- Candidate non-empty outputs: 20 / 20
- Candidate empty outputs: 0 / 20
- Candidate high-repetition rows: 2 / 20
- Candidate malformed-marker rows: 18 / 20
- Rows with >30% bbox-count delta: 14 / 20
- Average text similarity: 0.204
- Average reference elapsed: 9808 ms
- Average candidate elapsed: 4765 ms
- Average reference GPU after request: 31623 MB
- Average candidate GPU after request: 1678 MB
- Average reference bbox markers: 82.250
- Average candidate bbox markers: 23.300

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-suffix-ngram4-target-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.074 | 48 | 61 | 3944 | 0.312 | 13439 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.005 | 1 | 4 | 65 | 0.000 | 2298 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.015 | 1 | 5 | 337 | 0.000 | 2438 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.005 | 1 | 24 | 301 | 0.000 | 3242 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | grounding | 0.899 | 18 | 11 | 2016 | 0.000 | 2890 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0001-2200885e | plain_text | 0.029 | 18 | 22 | 1038 | 0.120 | 3759 | dataset/chinese-paper.pdf |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | ocr_boxes | 0.909 | 18 | 13 | 2107 | 0.000 | 2880 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0001-2200885e | document_parsing | 0.062 | 18 | 34 | 11807 | 0.886 | 12877 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | grounding | 0.119 | 48 | 80 | 10653 | 0.391 | 12956 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.035 | 305 | 8 | 305 | 0.085 | 2278 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
