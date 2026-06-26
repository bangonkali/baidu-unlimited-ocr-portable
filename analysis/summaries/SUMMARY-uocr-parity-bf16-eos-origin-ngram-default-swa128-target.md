# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T13:46:05+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-swa128-target`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-swa128-target.csv`
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
- Candidate high-repetition rows: 7 / 20
- Candidate malformed-marker rows: 3 / 20
- Rows with >30% bbox-count delta: 7 / 20
- Average text similarity: 0.508
- Average reference elapsed: 9568 ms
- Average candidate elapsed: 10615 ms
- Average reference GPU after request: 31639 MB
- Average candidate GPU after request: 1475 MB
- Average reference bbox markers: 76.700
- Average candidate bbox markers: 55.950

## Quality Finding

- `llamacpp-bf16-uocr-parity-eos-origin-ngram-default-swa128-target` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 21:19:00] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.035 | 48 | 1 | 6373 | 0.975 | 19427 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.019 | 1 | 1 | 8311 | 0.991 | 24901 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.019 | 1 | 1 | 8510 | 0.981 | 24874 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.013 | 1 | 0 | 8263 | 0.994 | 24874 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.168 | 305 | 45 | 1742 | 0.000 | 5642 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.126 | 272 | 45 | 1732 | 0.000 | 5711 | dataset/chinese-paper.pdf |
| candidate_repetition | upside-left-9e645a2a | grounding | 0.091 | 402 | 77 | 2082 | 0.412 | 7602 | dataset/upside left.jpg |
| low_similarity | upside-left-9e645a2a | plain_text | 0.014 | 4 | 11 | 450 | 0.128 | 3477 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | ocr_boxes | 0.037 | 294 | 380 | 6085 | 0.919 | 24496 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | document_parsing | 0.011 | 5 | 380 | 6082 | 0.923 | 24553 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
