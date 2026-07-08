#include "vl/paddle_ocr_vl_engine.hpp"

#include <chrono>
#include <string>
#include <utility>
#include <vector>

#include "vl/layout_analyzer.hpp"
#include "vl/llama_vlm_runner.hpp"
#include "vl/paddle_ocr_vl_result.hpp"

namespace agus_ocr {

class PaddleOcrVlEngine::Impl {
 public:
  Impl(const PaddleOcrVlBundleCheck& bundle,
       const agus_ocr_runtime_options_t& runtime)
      : layout_(bundle.layout_model_path, runtime),
        vlm_(bundle.vl_model_path, bundle.vl_mmproj_path, runtime) {}

  std::string Recognize(const agus_ocr_image_t& image,
                        const agus_ocr_run_options_t& options) {
    const auto total_start = std::chrono::steady_clock::now();
    PaddleOcrVlTiming timing;
    const cv::Mat page = DecodePaddleOcrVlImage(image);
    std::vector<LayoutRegion> regions;
    {
      const auto start = std::chrono::steady_clock::now();
      regions = layout_.Analyze(page);
      timing.detection_ms =
          PaddleOcrVlElapsedMs(start, std::chrono::steady_clock::now());
    }
    if (regions.empty()) {
      LayoutRegion region;
      region.label = "text";
      region.confidence = 1.0f;
      region.bounding_box =
          cv::Rect2f(0.0f, 0.0f, static_cast<float>(page.cols),
                     static_cast<float>(page.rows));
      region.polygon = {{0.0f, 0.0f},
                        {static_cast<float>(page.cols), 0.0f},
                        {static_cast<float>(page.cols),
                         static_cast<float>(page.rows)},
                        {0.0f, static_cast<float>(page.rows)}};
      regions.push_back(std::move(region));
    }

    VlGenerationOptions generation;
    generation.generate_markdown = options.generate_markdown != 0;
    generation.max_new_tokens =
        options.max_new_tokens > 0 ? options.max_new_tokens : 1024;
    generation.temperature = options.temperature;
    generation.min_pixels = options.min_pixels;
    generation.max_pixels =
        options.max_pixels > 0 ? options.max_pixels : generation.max_pixels;

    std::vector<RecognizedBlock> blocks;
    blocks.reserve(regions.size());
    for (size_t i = 0; i < regions.size(); ++i) {
      const cv::Rect crop_rect =
          ClampPaddleOcrVlCrop(regions[i].bounding_box, page.size());
      if (crop_rect.empty()) {
        continue;
      }
      const auto start = std::chrono::steady_clock::now();
      const std::string content =
          vlm_.Recognize(page(crop_rect).clone(), regions[i].label, generation);
      timing.recognition_ms +=
          PaddleOcrVlElapsedMs(start, std::chrono::steady_clock::now());

      RecognizedBlock block;
      block.region = regions[i];
      block.id = "block-" + std::to_string(i);
      block.text = content;
      block.markdown = generation.generate_markdown ? content : "";
      blocks.push_back(std::move(block));
    }

    timing.total_ms =
        PaddleOcrVlElapsedMs(total_start, std::chrono::steady_clock::now());
    return BuildPaddleOcrVlResultJson(page, blocks, timing, runtime_summary());
  }

  std::string runtime_summary() const {
    return "paddleocr-vl onnxruntime opencv " + vlm_.runtime_summary();
  }

 private:
  LayoutAnalyzer layout_;
  LlamaVlmRunner vlm_;
};

PaddleOcrVlEngine::PaddleOcrVlEngine(
    const PaddleOcrVlBundleCheck& bundle,
    const agus_ocr_runtime_options_t& runtime)
    : impl_(std::make_unique<Impl>(bundle, runtime)) {}

PaddleOcrVlEngine::~PaddleOcrVlEngine() = default;

std::string PaddleOcrVlEngine::Recognize(
    const agus_ocr_image_t& image,
    const agus_ocr_run_options_t& options) {
  return impl_->Recognize(image, options);
}

std::string PaddleOcrVlEngine::runtime_summary() const {
  return impl_->runtime_summary();
}

}  // namespace agus_ocr
