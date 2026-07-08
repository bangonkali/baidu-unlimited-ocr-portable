#ifndef TRAPO_OCR_VL_LLAMA_VLM_RUNNER_HPP_
#define TRAPO_OCR_VL_LLAMA_VLM_RUNNER_HPP_

#include <memory>
#include <string>
#include <vector>

#include "trapo_ocr.h"
#include "opencv2/core.hpp"

namespace trapo_ocr {

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
  trapo_ocr_generative_backend_t backend = TRAPO_OCR_GEN_BACKEND_CPU;
  bool supported = false;
  bool enabled_by_default = false;
  std::string device_name;
  std::string unavailable_reason;
  std::string last_failure;
};

std::vector<LlamaVlmBackendInfo> LlamaVlmBackendInfos();
trapo_ocr_generative_backend_t LlamaVlmDefaultBackend();

class LlamaVlmRunner {
 public:
  LlamaVlmRunner(const std::string& model_path,
                 const std::string& mmproj_path,
                 const trapo_ocr_runtime_options_t& runtime);
  ~LlamaVlmRunner();

  std::string Recognize(const cv::Mat& bgr, const std::string& label,
                        const VlGenerationOptions& options);
  std::string runtime_summary() const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_VL_LLAMA_VLM_RUNNER_HPP_
