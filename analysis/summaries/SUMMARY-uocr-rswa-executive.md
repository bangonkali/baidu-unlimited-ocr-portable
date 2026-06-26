# Unlimited-OCR R-SWA Executive Summary

Generated: 2026-06-27

## Scope

This summary covers the evaluation after factoring ggml-org/llama.cpp PR
#24975 style reference-SWA behavior into the local
`uocr-deepseek-ocr-parity` branch.

The work included:

- Core DeepSeek-OCR reference-SWA masking in llama.cpp.
- `n_ref` tracking across KV-cache lifecycle operations.
- Unlimited-OCR fallback `n_swa=128` when the current GGUF lacks
  `deepseek2-ocr.attention.sliding_window` metadata.
- Isolation of the older CLI KV-pruning SWA experiment behind
  `LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE=1`.

## Result Table

| Scenario | Summary | Pass | Empty | Repetition | Low sim | Avg sim | Avg candidate ms | GPU after MB | Read |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Previous Q4 CLI-prune baseline | `SUMMARY.md` pre-R-SWA section | 56 / 104 | 0 | 17 | 14 | 0.688 | 3809 | 1528 | Best historical zero-empty Q4 result, but used duplicate CLI KV pruning. |
| Q4 R-SWA default | `SUMMARY-uocr-rswa-q4-eos-origin-ngram-default-full.md` | 54 / 104 | 0 | 19 | 14 | 0.678 | 4216 | 2780 | Current practical Q4 code path; slightly worse than the historical Q4 baseline. |
| Q4 exact-prefill R-SWA | `SUMMARY-uocr-rswa-q4-noimgend-noeos-full.md` | 56 / 104 | 5 | 10 | 14 | 0.719 | 3311 | 2765 | Best average similarity, but empty rows make it a diagnostic path. |
| Q5_K_M R-SWA | `SUMMARY-uocr-rswa-q5_k_m-eos-origin-ngram-default-full.md` | 59 / 104 | 0 | 21 | 13 | 0.672 | 4426 | 2204 | Better pass count than Q4 R-SWA, worse repetition and similarity. |
| Q6_K R-SWA | `SUMMARY-uocr-rswa-q6_k-eos-origin-ngram-default-full.md` | 51 / 104 | 0 | 16 | 15 | 0.634 | 4211 | 2119 | Not useful as a quality improvement. |
| BF16 R-SWA | `SUMMARY-uocr-rswa-bf16-eos-origin-ngram-default-full.md` | 61 / 104 | 0 | 18 | 12 | 0.684 | 6963 | 3064 | Highest pass count so far; still not parity and too heavy for the default demo. |

## Decision

Overall parity did not improve enough for production packaging.

R-SWA is still worth keeping because it is the correct architectural direction
and it materially improves the BF16 quality ceiling over the prior BF16 run.
However, the practical Q4 demo default did not improve: it fell from 56 to 54
passes and average similarity fell from 0.688 to 0.678.

The current public demo should stay on the Q4 R-SWA native path because it is
zero-empty, fast enough for a demo, and uses the core runtime behavior rather
than the duplicate CLI KV-pruning experiment. BF16 is the best candidate if the
only objective is pass count, but it is slower and still fails 43 / 104 rows.

## Duplicate Implementation Decision

The older CLI `--deepseek-ocr-prefill-aware-swa` path previously removed
generated KV positions to approximate reference-SWA behavior. That is now a
duplicate of the core R-SWA implementation and is disabled by default.

Current behavior:

- `LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA=1` remains accepted for old scripts.
- Core R-SWA handles masking in llama.cpp.
- Legacy KV pruning only runs when
  `LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE=1` is explicitly set.

## Remaining Blocker

Prompt layout, local-grid composition, prefill token IDs, and first-token
top-k parity have already been validated on the smoke case. The remaining gap
is later generation behavior: logits/rank drift after generation starts still
causes layout, bbox, and repetition failures.
