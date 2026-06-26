# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:56:13+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-noimgend-target`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-eos-origin-ngram-default-noimgend-target.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_malformed_markers`: 1
- `candidate_repetition`: 10
- `low_similarity`: 1
- `pass`: 8

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
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-bf16-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram16-target.md`
- `SUMMARY-uocr-parity-q4-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-ngram-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-prefixnl-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram35-target-allprofiles.md`
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
- Candidate high-repetition rows: 10 / 20
- Candidate malformed-marker rows: 7 / 20
- Rows with >30% bbox-count delta: 5 / 20
- Average text similarity: 0.476
- Average reference elapsed: 9568 ms
- Average candidate elapsed: 7818 ms
- Average reference GPU after request: 31639 MB
- Average candidate GPU after request: 1465 MB
- Average reference bbox markers: 76.700
- Average candidate bbox markers: 120.150

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-noimgend-target` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 21:19:00] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.029 | 48 | 411 | 4453 | 0.937 | 13225 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.016 | 1 | 1 | 8253 | 0.986 | 13124 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.015 | 1 | 1 | 8235 | 0.989 | 13011 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.007 | 1 | 1 | 8185 | 0.990 | 13123 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | ocr_boxes | 0.944 | 18 | 17 | 2670 | 0.065 | 2993 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | grounding | 0.255 | 48 | 283 | 13140 | 0.859 | 12761 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.197 | 305 | 51 | 2098 | 0.121 | 3526 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.150 | 272 | 318 | 10039 | 0.801 | 12768 | dataset/chinese-paper.pdf |
| candidate_repetition | upside-left-9e645a2a | grounding | 0.041 | 402 | 362 | 2544 | 0.812 | 12317 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | plain_text | 0.009 | 4 | 406 | 5314 | 0.916 | 12263 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
