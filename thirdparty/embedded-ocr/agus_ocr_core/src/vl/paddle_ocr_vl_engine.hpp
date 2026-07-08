#ifndef AGUS_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_
#define AGUS_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_

#include <memory>
#include <string>

#include "agus_ocr.h"
#include "model/ocr_model_bundle.hpp"

namespace agus_ocr {

class PaddleOcrVlEngine {
 public:
  PaddleOcrVlEngine(const PaddleOcrVlBundleCheck& bundle,
                    const agus_ocr_runtime_options_t& runtime);
  ~PaddleOcrVlEngine();

  std::string Recognize(const agus_ocr_image_t& image,
                        const agus_ocr_run_options_t& options);
  std::string runtime_summary() const;

 private:
  class Impl;
  std::unique_ptr<Impl> impl_;
};

}  // namespace agus_ocr

#endif  // AGUS_OCR_VL_PADDLE_OCR_VL_ENGINE_HPP_
