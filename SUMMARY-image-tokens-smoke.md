# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T10:17:11+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-image-tokens-smoke`
- Metrics CSV: `results/compare/metrics.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `candidate_empty`: 1

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`

## Aggregate Metrics

- Reference result files: 1 / 1
- Candidate result files: 1 / 1
- Comparable pairs: 0 / 1
- Candidate non-empty outputs: 0 / 1
- Candidate empty outputs: 1 / 1
- Candidate high-repetition rows: 0 / 1
- Candidate malformed-marker rows: 0 / 1
- Rows with >30% bbox-count delta: 1 / 1
- Average text similarity: n/a
- Average reference elapsed: 4124 ms
- Average candidate elapsed: 1620 ms
- Average reference GPU after request: 31651 MB
- Average candidate GPU after request: 2067 MB
- Average reference bbox markers: 48.000
- Average candidate bbox markers: 0.000

## Quality Finding

- `llamacpp-q4_k_m-image-tokens-smoke` is not production-ready in this harness: most candidate rows are empty even though the process exits successfully.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 1620 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
