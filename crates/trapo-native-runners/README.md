# Trapo Native Runners

Small command-line wrappers that expose native OCR engines through the Trapo
runtime process contract. The binaries are packaged with each platform runtime
and exercised by the runtime engine guard before release.

The wrappers prefer packaged runtime payloads next to the selected runtime
`bin` directory:

- `tesseract/` contains `bin/tesseract(.exe)` and `tessdata/eng.traineddata`.
- `ppocrv6/` contains the PaddleOCR ONNXRuntime adapter, frozen engine binary,
  and pinned PP-OCRv6 model bundle.
