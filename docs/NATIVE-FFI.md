# Native FFI ABI

The portable workbench loads the native OCR runtime through the `uocr-ffi` ABI.
The Rust adapter lives under `src/trapo-server/src/ocr` and loads the platform
library from `thirdparty/uocr-runtime/<platform>/bin`.

Required ABI symbols are validated by:

```text
scripts/test_ctypes_runtime.py --abi-only
```

The runtime package workflow runs that ABI validation before publishing
`uocr-runtime-*` assets. Trapo release packaging then embeds the selected runtime
platforms into each portable app archive.
