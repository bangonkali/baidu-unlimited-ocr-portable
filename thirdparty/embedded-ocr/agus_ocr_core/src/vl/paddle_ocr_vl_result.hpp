#ifndef AGUS_OCR_VL_PADDLE_OCR_VL_RESULT_HPP_
#define AGUS_OCR_VL_PADDLE_OCR_VL_RESULT_HPP_

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

#include "agus_ocr.h"
#include "opencv2/core.hpp"
#include "vl/layout_analyzer.hpp"

namespace agus_ocr {

struct PaddleOcrVlTiming {
  int64_t detection_ms = 0;
  int64_t recognition_ms = 0;
  int64_t total_ms = 0;
};

struct RecognizedBlock {
  LayoutRegion region;
  std::string id;
  std::string text;
  std::string markdown;
};

cv::Mat DecodePaddleOcrVlImage(const agus_ocr_image_t& image);
cv::Rect ClampPaddleOcrVlCrop(const cv::Rect2f& box, const cv::Size& size);
int64_t PaddleOcrVlElapsedMs(std::chrono::steady_clock::time_point start,
                             std::chrono::steady_clock::time_point end);

std::string BuildPaddleOcrVlResultJson(
    const cv::Mat& page,
    const std::vector<RecognizedBlock>& blocks,
    const PaddleOcrVlTiming& timing,
    const std::string& runtime_summary);

}  // namespace agus_ocr

#endif  // AGUS_OCR_VL_PADDLE_OCR_VL_RESULT_HPP_
