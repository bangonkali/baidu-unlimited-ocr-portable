# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T10:17:09+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q5_k_m`
- Metrics CSV: `results/compare/metrics.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_empty`: 6
- `candidate_repetition`: 3

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-prompts.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q6_k.md`

## Aggregate Metrics

- Reference result files: 10 / 10
- Candidate result files: 10 / 10
- Comparable pairs: 4 / 10
- Candidate non-empty outputs: 4 / 10
- Candidate empty outputs: 6 / 10
- Candidate high-repetition rows: 3 / 10
- Candidate malformed-marker rows: 2 / 10
- Rows with >30% bbox-count delta: 10 / 10
- Average text similarity: 0.549
- Average reference elapsed: 9043 ms
- Average candidate elapsed: 4006 ms
- Average reference GPU after request: 31672 MB
- Average candidate GPU after request: 2755 MB
- Average reference bbox markers: 86.200
- Average candidate bbox markers: 58.700

## Quality Finding

- `llamacpp-q5_k_m` is not production-ready in this harness: most candidate rows are empty even though the process exits successfully.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | grounding | n/a | 48 | 0 | 0 | 0.000 | 3744 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_empty | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | plain_text | n/a | 1 | 0 | 0 | 0.000 | 1580 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | chinese-paper-page-0001-2200885e | grounding | 0.482 | 18 | 436 | 5029 | 0.528 | 11873 | dataset/chinese-paper.pdf |
| bbox_count_mismatch | chinese-paper-page-0001-2200885e | plain_text | 0.761 | 18 | 31 | 2907 | 0.063 | 2820 | dataset/chinese-paper.pdf |
| candidate_empty | chinese-paper-page-0002-3d10e38a | grounding | n/a | 48 | 0 | 0 | 0.000 | 1456 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | plain_text | 0.168 | 305 | 119 | 9615 | 0.502 | 11731 | dataset/chinese-paper.pdf |
| candidate_empty | sc-02-45a8efac | grounding | n/a | 9 | 0 | 0 | 0.000 | 1297 | dataset/sc-02.png |
| candidate_repetition | sc-02-45a8efac | plain_text | 0.783 | 9 | 1 | 2988 | 0.452 | 2243 | dataset/sc-02.png |
| candidate_empty | upside-left-9e645a2a | grounding | n/a | 402 | 0 | 0 | 0.000 | 1635 | dataset/upside left.jpg |
| candidate_empty | upside-left-9e645a2a | plain_text | n/a | 4 | 0 | 0 | 0.000 | 1681 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
