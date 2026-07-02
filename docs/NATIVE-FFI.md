# Native FFI ABI

Trapo loads local OCR runtimes through the `uocr-ffi` ABI. The ABI name is kept
stable so runtime archives can evolve independently from the Trapo Rust server
and React workbench.

The Rust adapter lives in:

```text
src/trapo-server/src/ocr
```

At startup and during release smoke tests, Trapo validates the ABI version and
required symbols before creating an OCR session. Runtime package CI also checks
the exported ABI with:

```text
scripts/test_ctypes_runtime.py --abi-only
```

macOS packages additionally validate that runtime dylibs use portable loader
paths and do not depend on Homebrew-only absolute library paths.
