# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T11:53:22+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-gundam-exact-full`
- Metrics CSV: `results/compare/SUMMARY-deepseekocr-gundam-exact-full.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_empty`: 36
- `candidate_malformed_markers`: 1
- `candidate_repetition`: 5
- `pass`: 8
- `review`: 1

## Related Strategy Summaries

- `SUMMARY-bf16.md`
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

## Aggregate Metrics

- Reference result files: 52 / 52
- Candidate result files: 52 / 52
- Comparable pairs: 16 / 52
- Candidate non-empty outputs: 16 / 52
- Candidate empty outputs: 36 / 52
- Candidate high-repetition rows: 5 / 52
- Candidate malformed-marker rows: 2 / 52
- Rows with >30% bbox-count delta: 42 / 52
- Average text similarity: 0.691
- Average reference elapsed: 6239 ms
- Average candidate elapsed: 3170 ms
- Average reference GPU after request: 31674 MB
- Average candidate GPU after request: 2885 MB
- Average reference bbox markers: 70.981
- Average candidate bbox markers: 20.385

## Quality Finding

- `llamacpp-q4_k_m-gundam-exact-full` is not production-ready in this harness: most candidate rows are empty even though the process exits successfully.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 2562 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 2200 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | grounding | n/a | 1 | 0 | 0 | 0.000 | 2311 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | 613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5-cf68e1dc | plain_text | n/a | 307 | 0 | 0 | 0.000 | 2271 | dataset/613257452-2dc44c5b-a2b5-4366-ba87-d3b86bab16d5.png |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 1979 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | plain_text | n/a | 305 | 0 | 0 | 0.000 | 1960 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0004-5ff6d97f | plain_text | 0.000 | 17 | 0 | 29487 | 0.999 | 12950 | dataset/chinese-paper.pdf |
| candidate_malformed_markers | chinese-paper-page-0005-293c62b2 | plain_text | 0.995 | 11 | 9 | 4262 | 0.000 | 3074 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0006-5340e758 | grounding | n/a | 11 | 0 | 0 | 0.000 | 2063 | dataset/chinese-paper.pdf |
| candidate_empty | content-40d41a0b | grounding | n/a | 17 | 0 | 0 | 0.000 | 1705 | dataset/content.png |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
