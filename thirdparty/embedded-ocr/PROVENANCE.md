# embedded-ocr provenance

The former native OCR core was copied from the internal prototype:

- Source path: `C:\Users\Bangonkali\Desktop\Projects\embedded-ocr\native\agus_ocr_core`
- Imported for: native `trapo-ocr-ffi` implementation work covering PP-OCRv6 ONNX/OpenCV/Clipper and in-process llama.cpp/mtmd PaddleOCR-VL GGUF execution.

That code is now Trapo-owned source under `src/trapo-ocr-native`. It is no
longer treated as an external thirdparty source tree. The remaining files under
`thirdparty/embedded-ocr` are provenance/model-asset references only.

The sibling `embedded-ocr` repository is an internal reference. Its top-level
license is currently a placeholder, so the copied implementation remains treated
as internal/reference-derived code until an explicit project license decision is
recorded before shipping.

The bundled Clipper files keep their own Boost Software License notice at
`src/trapo-ocr-native/third_party/clipper/LICENSE`.
