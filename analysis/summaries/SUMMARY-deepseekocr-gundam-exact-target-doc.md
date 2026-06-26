# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T11:39:58+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-gundam-exact-target-doc`
- Metrics CSV: `results/compare/SUMMARY-deepseekocr-gundam-exact-target-doc.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_repetition`: 2
- `pass`: 3

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-deepseekocr-gundam-exact-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-exact-smoke.md`
- `SUMMARY-deepseekocr-gundam-rp105-smoke.md`
- `SUMMARY-deepseekocr-gundam-smoke.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-prompts-sc02-document.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`

## Aggregate Metrics

- Reference result files: 5 / 5
- Candidate result files: 5 / 5
- Comparable pairs: 5 / 5
- Candidate non-empty outputs: 5 / 5
- Candidate empty outputs: 0 / 5
- Candidate high-repetition rows: 2 / 5
- Candidate malformed-marker rows: 0 / 5
- Rows with >30% bbox-count delta: 1 / 5
- Average text similarity: 0.568
- Average reference elapsed: 8789 ms
- Average candidate elapsed: 7053 ms
- Average reference GPU after request: 31574 MB
- Average candidate GPU after request: 2902 MB
- Average reference bbox markers: 15.600
- Average candidate bbox markers: 16.400

## Quality Finding

- `llamacpp-q4_k_m-gundam-exact-target-doc` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.007 | 1 | 1 | 8218 | 0.998 | 13407 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | upside-left-9e645a2a | document_parsing | 0.001 | 5 | 3 | 28497 | 0.999 | 12480 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
