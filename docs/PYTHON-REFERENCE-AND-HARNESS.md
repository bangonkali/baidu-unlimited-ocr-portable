# Python Reference App And Validation Harness

The C++/React workbench is the target product path. The Python Gradio app and
analysis harness remain useful for behavior comparison, OCR strategy research,
and runtime parity checks. They are secondary workflows and are not required to
run the Windows portable exe.

## Python Reference App

The reference app lives in:

```text
src/baidu_unlimited_ocr_portable/
```

It can launch the native Unlimited-OCR runtime through Python bindings or
subprocess tooling and remains the behavioral reference for OCR profiles,
marker parsing, PDF/image intake behavior, and comparison against historical
results.

Run a short smoke from the repository root:

```powershell
uv run --project unlimited-ocr-portable baidu-uocr-client `
  --smoke --image dataset/sc-02.png --max-tokens 64
```

Launch the reference UI:

```powershell
uv run --project unlimited-ocr-portable baidu-uocr-client `
  --host 127.0.0.1 --port 7861
```

The Python path may use Python packages, Gradio, and research-only runtime
switches. Those dependencies are intentionally not part of the launched
`uocr-server.exe` product.

## Harness

The validation harness lives under:

```text
analysis/uocr_harness/
analysis/summaries/
```

Typical commands:

```powershell
uv run --project unlimited-ocr-portable uocr-harness prepare
uv run --project unlimited-ocr-portable uocr-harness run-llamacpp --limit 1 --max-tokens 256
uv run --project unlimited-ocr-portable uocr-harness compare
```

For full validation procedures and result interpretation, use:

```text
TEST-PROCEDURE.md
analysis/summaries/SUMMARY.md
analysis/summaries/SUMMARY-uocr-rswa-executive.md
```

The current practical Q4 profile is still the app default:

```text
best-zero-empty-q4
```

The diagnostic profile remains:

```text
experimental-exact-prefill-q4
```

## When To Use This Path

Use the Python/reference path when you need to:

- compare C++ behavior against the historical Gradio demo,
- reproduce OCR profile experiments,
- inspect SGLang or llama.cpp parity artifacts,
- run harness summaries, or
- debug a model/runtime issue outside the portable workbench.

Use the Windows portable workbench for normal OCR usage.
