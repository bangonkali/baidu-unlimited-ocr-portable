# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T16:44:37+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-full`
- Metrics CSV: `results/compare/SUMMARY-uocr-parity-q4-noimgend-noeos-full.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 10
- `candidate_empty`: 5
- `candidate_repetition`: 27
- `low_similarity`: 6
- `pass`: 49
- `review`: 7

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-exact-full.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-target-doc.md`
- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-target-doc.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-generation-steps-bf16-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q4-noimgend-noeos-noswa-64tok.md`
- `SUMMARY-generation-steps-q5-noimgend-noeos-64tok.md`
- `SUMMARY-generation-steps-q6-noimgend-noeos-64tok.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-parity-artifacts-native-onetok.md`
- `SUMMARY-parity-artifacts-noimgend-smoke.md`
- `SUMMARY-parity-artifacts-smoke.md`
- `SUMMARY-q4-prompts-sc02-document.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`
- `SUMMARY-runtime-parity-noeos-smoke.md`
- `SUMMARY-runtime-parity-noimgend-noeos-smoke.md`
- `SUMMARY-runtime-parity-noimgend-smoke.md`
- `SUMMARY-runtime-parity-smoke.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-bf16-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-bf16-grid-allprofiles.md`
- `SUMMARY-uocr-parity-noimgend-noeos-smoke.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-noimgend-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-swa128-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-eos-origin-ngram16-target.md`
- `SUMMARY-uocr-parity-q4-grid-allprofiles.md`
- `SUMMARY-uocr-parity-q4-ngram-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-target.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-minnew1-target.md`
- `SUMMARY-uocr-parity-q4-origin-ngram-default-target.md`
- `SUMMARY-uocr-parity-q4-prefixnl-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram35-target-allprofiles.md`
- `SUMMARY-uocr-parity-q4-suffix-ngram4-target-allprofiles.md`
- `SUMMARY-uocr-placement-auto.md`
- `SUMMARY-uocr-placement-prefix-newline.md`
- `SUMMARY-uocr-placement-prefix-tight.md`
- `SUMMARY-uocr-placement-suffix-newline.md`

## Aggregate Metrics

- Reference result files: 104 / 104
- Candidate result files: 104 / 104
- Comparable pairs: 97 / 104
- Candidate non-empty outputs: 99 / 104
- Candidate empty outputs: 5 / 104
- Candidate high-repetition rows: 27 / 104
- Candidate malformed-marker rows: 23 / 104
- Rows with >30% bbox-count delta: 36 / 104
- Average text similarity: 0.671
- Average reference elapsed: 5833 ms
- Average candidate elapsed: 4719 ms
- Average reference GPU after request: 31639 MB
- Average candidate GPU after request: 2440 MB
- Average reference bbox markers: 70.135
- Average candidate bbox markers: 64.529

## Quality Finding

- `llamacpp-q4_k_m-uocr-parity-noimgend-noeos-full` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-27 00:29:00] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.032 | 48 | 40 | 25423 | 0.912 | 13446 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.019 | 1 | 1 | 8229 | 0.989 | 13116 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.016 | 1 | 1 | 8253 | 0.987 | 13312 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.006 | 1 | 1 | 8186 | 0.990 | 13113 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| low_similarity | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | 0.008 | 1 | 2 | 147 | 0.000 | 2218 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | 0.040 | 307 | 405 | 4697 | 0.880 | 13046 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| bbox_count_mismatch | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | ocr_boxes | 0.623 | 1 | 2 | 147 | 0.000 | 2207 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | document_parsing | 0.054 | 405 | 97 | 12713 | 0.847 | 13001 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | grounding | 0.198 | 48 | 299 | 12024 | 0.875 | 12822 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | plain_text | 0.175 | 305 | 297 | 12667 | 0.840 | 12629 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
