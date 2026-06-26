# Unlimited-OCR Portable Validation Summary

Generated: 2026-06-26T10:17:10+00:00

## Engines

- Reference: `sglang`
- Candidate: `llamacpp-q4_k_m-prompts`
- Metrics CSV: `results/compare/metrics.csv`
- Test procedure: `TEST-PROCEDURE.md`

## Status Counts

- `bbox_count_mismatch`: 1
- `candidate_malformed_markers`: 1
- `candidate_repetition`: 8

## Related Strategy Summaries

- `SUMMARY-bf16.md`
- `SUMMARY-image-tokens-smoke.md`
- `SUMMARY-llamacpp-server-q4.md`
- `SUMMARY-q4-rp105.md`
- `SUMMARY-q5_k_m.md`
- `SUMMARY-q6_k.md`

## Aggregate Metrics

- Reference result files: 10 / 10
- Candidate result files: 10 / 10
- Comparable pairs: 10 / 10
- Candidate non-empty outputs: 10 / 10
- Candidate empty outputs: 0 / 10
- Candidate high-repetition rows: 8 / 10
- Candidate malformed-marker rows: 2 / 10
- Rows with >30% bbox-count delta: 5 / 10
- Average text similarity: 0.203
- Average reference elapsed: 10574 ms
- Average candidate elapsed: 10593 ms
- Average reference GPU after request: 31574 MB
- Average candidate GPU after request: 2053 MB
- Average reference bbox markers: 78.300
- Average candidate bbox markers: 131.600

## Quality Finding

- `llamacpp-q4_k_m-prompts` needs more validation before packaging: non-empty outputs still show repetition or layout-marker mismatches.

## Reference Runner Status

- SGLang log exists. Last line: `[2026-06-26 18:05:55] Gracefully exiting... Remaining number of requests 0. Remaining requests remaining_rids=[].`

## Review Queue

| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | ocr_boxes | 0.000 | 1 | 1 | 46334 | 0.999 | 11678 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| candidate_repetition | 613256554-0d2742ca-467c-4b2b-8294-20c07609e316-1db675f4 | document_parsing | 0.000 | 1 | 1 | 12268 | 0.998 | 11422 | dataset/613256554-0d2742ca-467c-4b2b-8294-20c07609e316.png |
| bbox_count_mismatch | chinese-paper-page-0001-2200885e | ocr_boxes | 0.822 | 18 | 31 | 2717 | 0.059 | 2647 | dataset/chinese-paper.pdf |
| candidate_malformed_markers | chinese-paper-page-0001-2200885e | document_parsing | 0.704 | 18 | 474 | 2452 | 0.060 | 11435 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | ocr_boxes | 0.138 | 327 | 310 | 12751 | 0.806 | 11459 | dataset/chinese-paper.pdf |
| candidate_repetition | chinese-paper-page-0002-3d10e38a | document_parsing | 0.309 | 45 | 107 | 9465 | 0.512 | 11414 | dataset/chinese-paper.pdf |
| candidate_repetition | sc-02-45a8efac | ocr_boxes | 0.011 | 9 | 7 | 16225 | 0.774 | 11301 | dataset/sc-02.png |
| candidate_repetition | sc-02-45a8efac | document_parsing | 0.011 | 9 | 8 | 16196 | 0.773 | 11321 | dataset/sc-02.png |
| candidate_repetition | upside-left-9e645a2a | ocr_boxes | 0.016 | 350 | 13 | 26490 | 0.989 | 11616 | dataset/upside left.jpg |
| candidate_repetition | upside-left-9e645a2a | document_parsing | 0.013 | 5 | 364 | 8004 | 0.942 | 11636 | dataset/upside left.jpg |

## Notes

- `candidate_empty` means the process completed but normalized output text was empty.
- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.
- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.
- Similarity is computed after removing detection/ref markers and coordinates.
- Bounding-box quality still needs visual review for cases with marker count mismatches.
