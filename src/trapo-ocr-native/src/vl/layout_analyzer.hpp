#ifndef TRAPO_OCR_VL_LAYOUT_ANALYZER_HPP_
#define TRAPO_OCR_VL_LAYOUT_ANALYZER_HPP_

#include <string>
#include <memory>
#include <vector>

#include "trapo_ocr.h"
#include "opencv2/core.hpp"

namespace trapo_ocr {

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
                 const trapo_ocr_runtime_options_t& runtime);

  std::vector<LayoutRegion> Analyze(const cv::Mat& bgr) const;

 private:
  class Impl;
  std::shared_ptr<Impl> impl_;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_VL_LAYOUT_ANALYZER_HPP_
