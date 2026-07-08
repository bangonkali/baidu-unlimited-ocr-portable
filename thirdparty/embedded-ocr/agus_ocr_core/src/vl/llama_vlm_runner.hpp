#ifndef AGUS_OCR_VL_LLAMA_VLM_RUNNER_HPP_
#define AGUS_OCR_VL_LLAMA_VLM_RUNNER_HPP_

#include <memory>
#include <string>
#include <vector>

#include "agus_ocr.h"
#include "opencv2/core.hpp"

namespace agus_ocr {

struct VlGenerationOptions {
  bool generate_markdown = true;
  int max_new_tokens = 1024;
  float temperature = 0.0f;
  int min_pixels = 0;
  int max_pixels = 1003520;
};

bool LlamaVlmRuntimeAvailable();
std::string LlamaVlmUnavailableReason();

struct LlamaVlmBackendInfo {
  agus_ocr_generative_backend_t backend = AGUS_OCR_GEN_BACKEND_CPU;
  bool supported = false;
  bool enabled_by_default = false;
  std::string device_name;
  std::string unavailable_reason;
  std::string last_failure;
};

std::vector<LlamaVlmBackendInfo> LlamaVlmBackendInfos();
agus_ocr_generative_backend_t LlamaVlmDefaultBackend();

class LlamaVlmRunner {
 public:
  LlamaVlmRunner(const std::string& model_path,
                 const std::string& mmproj_path,
                 const agus_ocr_runtime_options_t& runtime);
  ~LlamaVlmRunner();

  std::string Recognize(const cv::Mat& bgr, const std::string& label,
                        const VlGenerationOptions& options);
  std::string runtime_summary() const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

}  // namespace agus_ocr

#endif  // AGUS_OCR_VL_LLAMA_VLM_RUNNER_HPP_
