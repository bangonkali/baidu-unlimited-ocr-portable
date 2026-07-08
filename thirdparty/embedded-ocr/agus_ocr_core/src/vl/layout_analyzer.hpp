#ifndef AGUS_OCR_VL_LAYOUT_ANALYZER_HPP_
#define AGUS_OCR_VL_LAYOUT_ANALYZER_HPP_

#include <string>
#include <memory>
#include <vector>

#include "agus_ocr.h"
#include "opencv2/core.hpp"

namespace agus_ocr {

struct LayoutRegion {
  std::string label;
  float confidence = 0.0f;
  int reading_order = 0;
  std::vector<cv::Point2f> polygon;
  cv::Rect2f bounding_box;
};

class LayoutAnalyzer {
 public:
  LayoutAnalyzer(const std::string& model_path,
                 const agus_ocr_runtime_options_t& runtime);

  std::vector<LayoutRegion> Analyze(const cv::Mat& bgr) const;

 private:
  class Impl;
  std::shared_ptr<Impl> impl_;
};

}  // namespace agus_ocr

#endif  // AGUS_OCR_VL_LAYOUT_ANALYZER_HPP_
