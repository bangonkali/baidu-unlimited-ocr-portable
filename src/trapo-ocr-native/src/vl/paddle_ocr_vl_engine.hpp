#ifndef TRAPO_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_
#define TRAPO_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_

#include <memory>
#include <string>

#include "trapo_ocr.h"
#include "model/ocr_model_bundle.hpp"

namespace trapo_ocr {

class PaddleOcrVlEngine {
 public:
  PaddleOcrVlEngine(const PaddleOcrVlBundleCheck& bundle,
                    const trapo_ocr_runtime_options_t& runtime);
  ~PaddleOcrVlEngine();

  std::string Recognize(const trapo_ocr_image_t& image,
                        const trapo_ocr_run_options_t& options);
  std::string runtime_summary() const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_
