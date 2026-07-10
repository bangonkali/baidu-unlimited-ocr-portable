#pragma once

#include <onnxruntime_cxx_api.h>

namespace trapo_ocr {

// Appends the ONNX Runtime CUDA execution provider. Throws on missing API or
// provider failure. Supported on Windows and Linux cuda13 hosts.
void AppendOrtCudaExecutionProvider(Ort::SessionOptions& options,
                                    int device_id = 0);

}  // namespace trapo_ocr
