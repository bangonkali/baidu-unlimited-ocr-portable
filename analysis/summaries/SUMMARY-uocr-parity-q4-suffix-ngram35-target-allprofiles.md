# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:07:05+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-suffix-ngram35-target-allprofiles`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-suffix-ngram35-target-allprofiles.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_repetition`: 15
- `low_similarity`: 2
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
- `SUMMARY-uocr-parity-q4-suffix-ngram4-target-allprofiles.md`
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
- Candidate high-repetition rows: 15 / 20
- Candidate malformed-marker rows: 10 / 20
- Rows with >30% bbox-count delta: 12 / 20
- Average text similarity: 0.194
- Average reference elapsed: 9808 ms
- Average candidate elapsed: 8904 ms
- Average reference GPU after request: 31623 MB
- Average candidate GPU after request: 1697 MB
- Average reference bbox markers: 82.250
- Average candidate bbox markers: 125.050

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-suffix-ngram35-target-allprofiles` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 20:24:17] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| low_similarity | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.215 | 48 | 15 | 667 | 0.328 | 3310 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.003 | 1 | 136 | 3353 | 0.801 | 8195 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.026 | 1 | 1 | 193 | 0.457 | 2304 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.005 | 1 | 73 | 10667 | 0.984 | 13134 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | chinese-paper-page-0001-2200885e | plain_text | 0.026 | 18 | 197 | 10523 | 0.639 | 12883 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0001-2200885e | document_parsing | 0.018 | 18 | 24 | 334 | 0.500 | 2628 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | plain_text | 0.050 | 305 | 339 | 11652 | 0.505 | 12757 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.337 | 327 | 315 | 10271 | 0.861 | 12819 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | document_parsing | 0.148 | 45 | 414 | 7160 | 0.839 | 12885 | dataset/chinese-paper.pdf |
| candidate_repetition | sc-02-45a8efac | grounding | 0.130 | 9 | 9 | 31197 | 0.940 | 12025 | dataset/sc-02.png |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
