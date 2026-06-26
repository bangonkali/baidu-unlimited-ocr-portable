# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:36:35+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-target`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-target.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_repetition`: 7
- `low_similarity`: 3
- `pass`: 10

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
- Candidate high-repetition rows: 7 / 20
- Candidate malformed-marker rows: 4 / 20
- Rows with >30% bbox-count delta: 7 / 20
- Average text similarity: 0.513
- Average reference elapsed: 9568 ms
- Average candidate elapsed: 12899 ms
- Average reference GPU after request: 31639 MB
- Average candidate GPU after request: 1475 MB
- Average reference bbox markers: 76.700
- Average candidate bbox markers: 69.700

## Quality Finding

- `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-target` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 21:19:00] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.026 | 48 | 1 | 8452 | 0.980 | 25510 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.019 | 1 | 1 | 8311 | 0.986 | 25046 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.019 | 1 | 1 | 8509 | 0.977 | 25150 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.013 | 1 | 0 | 8262 | 0.990 | 24970 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.226 | 305 | 50 | 2100 | 0.091 | 6086 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.190 | 272 | 317 | 10301 | 0.835 | 24815 | dataset/chinese-paper.pdf |
| low_similarity | upside-left-9e645a2a | grounding | 0.070 | 402 | 36 | 802 | 0.344 | 4622 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | plain_text | 0.016 | 4 | 389 | 10002 | 0.944 | 24318 | dataset/upside left.jpg |
| low_similarity | upside-left-9e645a2a | ocr_boxes | 0.017 | 294 | 8 | 20679 | 0.000 | 24358 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | document_parsing | 0.011 | 5 | 381 | 6706 | 0.928 | 24432 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
