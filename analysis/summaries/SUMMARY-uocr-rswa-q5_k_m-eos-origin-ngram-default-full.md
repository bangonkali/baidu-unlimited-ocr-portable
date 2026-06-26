# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T19:22:17+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q5_k_m-uocr-rswa-eos-origin-ngram-default-full`
- Metrics CSV: `/home/ubuntu/projects/unlimited-ocr/unlimited-ocr-portable/results/compare/SUMMARY-uocr-rswa-q5_k_m-eos-origin-ngram-default-full.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 5
- `candidate_malformed_markers`: 1
- `candidate_repetition`: 21
- `low_similarity`: 13
- `pass`: 59
- `review`: 5

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
- `SUMMARY-parity-artifacts-output-embeddings-onetok.md`
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
- `SUMMARY-uocr-parity-q4-noimgend-noeos-full.md`
- `SUMMARY-uocr-parity-q4-noimgend-noeos-swa128-full.md`
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
- `SUMMARY-uocr-rswa-bf16-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-q4-eos-origin-ngram-default-full.md`
- `SUMMARY-uocr-rswa-q4-noimgend-noeos-full.md`

## Aggregate Metrics

- Reference result files: 104 / 104
- Candidate result files: 104 / 104
- Comparable pairs: 99 / 104
- Candidate non-empty outputs: 104 / 104
- Candidate empty outputs: 0 / 104
- Candidate high-repetition rows: 21 / 104
- Candidate malformed-marker rows: 11 / 104
- Rows with >30% bbox-count delta: 38 / 104
- Average text similarity: 0.672
- Average reference elapsed: 5833 ms
- Average candidate elapsed: 4426 ms
- Average reference GPU after request: 31639 MB
- Average candidate GPU after request: 2204 MB
- Average reference bbox markers: 70.135
- Average candidate bbox markers: 37.913

## Quality Finding

- `llamacpp-q5_k_m-uocr-rswa-eos-origin-ngram-default-full` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

No SGLang startup error log found.

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | 0.044 | 48 | 34 | 27945 | 0.923 | 23970 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | 0.004 | 1 | 10 | 44937 | 0.971 | 18081 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.006 | 1 | 131 | 19841 | 0.896 | 13394 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.006 | 1 | 0 | 8221 | 0.995 | 13394 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | 0.012 | 1 | 127 | 5625 | 0.824 | 7737 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | 0.341 | 307 | 67 | 991 | 0.487 | 3920 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | ocr_boxes | 0.218 | 1 | 67 | 987 | 0.587 | 3959 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_repetition | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | document_parsing | 0.009 | 405 | 1 | 8217 | 0.995 | 13491 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| low_similarity | chinese-paper-page-0002-3d10e38a | plain_text | 0.168 | 305 | 45 | 1732 | 0.000 | 3486 | dataset/chinese-paper.pdf |
| low_similarity | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.126 | 272 | 45 | 1732 | 0.000 | 3466 | dataset/chinese-paper.pdf |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
