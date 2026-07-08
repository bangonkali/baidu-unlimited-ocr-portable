#ifndef AGUS_OCR_GEMMA_IMAGE_PROCESSOR_HPP_
#define AGUS_OCR_GEMMA_IMAGE_PROCESSOR_HPP_

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#include <opencv2/core.hpp>

#include "agus_ocr.h"
#include "gemma/gemma_onnx_runner.hpp"
#include "model/ocr_model_bundle.hpp"

namespace agus_ocr {

struct GemmaImageInputs {
  cv::Mat source_page;
  cv::Mat oriented_page;
  int doc_angle = -1;
  int soft_tokens = 0;
  int max_patches = 0;
  std::vector<float> pixel_values;
  std::vector<int64_t> pixel_shape;
  std::vector<int64_t> position_ids;
  std::vector<int64_t> position_shape;
  std::string overlay_mime_type;
  std::string overlay_image_base64;
};

class GemmaImageProcessor {
 public:
  GemmaImageProcessor(const GemmaMarkdownBundleCheck& bundle,
                      agus_ocr_backend_t backend,
                      int32_t cpu_threads,
                      bool enable_profiling);

  GemmaImageInputs Process(const agus_ocr_image_t& image,
                           int32_t visual_token_budget,
                           bool use_doc_orientation) const;

 private:
  int DetectOrientation(const cv::Mat& page) const;

  GemmaOnnxSession doc_orientation_;
};

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_IMAGE_PROCESSOR_HPP_
