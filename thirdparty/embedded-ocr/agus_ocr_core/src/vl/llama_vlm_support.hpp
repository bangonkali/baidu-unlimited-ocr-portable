#ifndef AGUS_OCR_VL_LLAMA_VLM_SUPPORT_HPP_
#define AGUS_OCR_VL_LLAMA_VLM_SUPPORT_HPP_

#include <string>

#include "agus_ocr.h"
#include "opencv2/core.hpp"
#include "vl/llama_vlm_runner.hpp"

namespace agus_ocr {

int ResolveLlamaThreads(const agus_ocr_runtime_options_t& runtime);
cv::Mat ResizeToPixelBudget(const cv::Mat& bgr,
                            const VlGenerationOptions& options);
std::string PromptForLayoutLabel(const std::string& label, bool markdown);
std::string TrimGeneratedText(std::string value);

}  // namespace agus_ocr

#endif  // AGUS_OCR_VL_LLAMA_VLM_SUPPORT_HPP_
