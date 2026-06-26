# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T10:17:10+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q6_k`
- Metrics CSV: `results/compare/metrics.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_empty`: 6
- `candidate_malformed_markers`: 1
- `candidate_repetition`: 2

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`

## Aggregate Metrics

- Reference result files: 10 / 10
- Candidate result files: 10 / 10
- Comparable pairs: 4 / 10
- Candidate non-empty outputs: 4 / 10
- Candidate empty outputs: 6 / 10
- Candidate high-repetition rows: 2 / 10
- Candidate malformed-marker rows: 2 / 10
- Rows with >30% bbox-count delta: 9 / 10
- Average text similarity: 0.431
- Average reference elapsed: 9043 ms
- Average candidate elapsed: 4218 ms
- Average reference GPU after request: 31672 MB
- Average candidate GPU after request: 2762 MB
- Average reference bbox markers: 86.200
- Average candidate bbox markers: 36.900

## Quality Finding

- `llamacpp-q6_k` is not production-ready in this harness: most candidate rows are empty even though the process exits successfully.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 4225 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 1623 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | grounding | 0.801 | 18 | 29 | 2696 | 0.059 | 2947 | dataset/chinese-paper.pdf |
| bbox_count_mismatch | chinese-paper-page-0001-2200885e | plain_text | 0.774 | 18 | 32 | 2848 | 0.057 | 2893 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 1456 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | plain_text | 0.148 | 305 | 308 | 9847 | 0.697 | 12234 | dataset/chinese-paper.pdf |
| candidate_repetition | sc-02-45a8efac | grounding | 0.000 | 9 | 0 | 38619 | 0.999 | 12006 | dataset/sc-02.png |
| candidate_empty | sc-02-45a8efac | plain_text | n/a | 9 | 0 | 0 | 0.000 | 1359 | dataset/sc-02.png |
| candidate_empty | upside-left-9e645a2a | grounding | n/a | 402 | 0 | 0 | 0.000 | 1688 | dataset/upside left.jpg |
| candidate_empty | upside-left-9e645a2a | plain_text | n/a | 4 | 0 | 0 | 0.000 | 1752 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
