#include "ort/ort_cuda_ep.hpp"

#include <stdexcept>

namespace trapo_ocr {

void AppendOrtCudaExecutionProvider(Ort::SessionOptions& options,
                                    int device_id) {
#if defined(_WIN32) || defined(__linux__)
  OrtCUDAProviderOptions cuda_options{};
  cuda_options.device_id = device_id;
  const auto append_cuda =
      Ort::GetApi().SessionOptionsAppendExecutionProvider_CUDA;
  if (append_cuda == nullptr) {
    throw std::runtime_error(
        "ONNX Runtime CUDA was selected but this build does not expose the "
        "CUDA provider API");
  }
  Ort::ThrowOnError(append_cuda(options, &cuda_options));
#else
  (void)options;
  (void)device_id;
  throw std::runtime_error(
      "ONNX Runtime CUDA is only available on Windows and Linux hosts");
#endif
}

}  // namespace trapo_ocr
