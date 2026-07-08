#ifndef TRAPO_OCR_VL_LLAMA_VLM_SUPPORT_HPP_
#define TRAPO_OCR_VL_LLAMA_VLM_SUPPORT_HPP_

#include <string>

#include "trapo_ocr.h"
#include "opencv2/core.hpp"
#include "vl/llama_vlm_runner.hpp"

namespace trapo_ocr {

int ResolveLlamaThreads(const trapo_ocr_runtime_options_t& runtime);
cv::Mat ResizeToPixelBudget(const cv::Mat& bgr,
                            const VlGenerationOptions& options);
std::string PromptForLayoutLabel(const std::string& label, bool markdown);
std::string TrimGeneratedText(std::string value);

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_VL_LLAMA_VLM_SUPPORT_HPP_
