#include "vl/layout_analyzer.hpp"

#include <algorithm>
#include <cmath>
#include <memory>
#include <numeric>
#include <iterator>
#include <stdexcept>

#include "onnxruntime_cxx_api.h"
#include "opencv2/imgproc.hpp"

#if defined(_WIN32)
#include <windows.h>
#endif

namespace trapo_ocr {
namespace {

constexpr int kLayoutInputSize = 800;
constexpr float kLayoutThreshold = 0.5f;

const char* kLayoutLabels[] = {
    "abstract",       "algorithm",    "aside_text",       "chart",
    "content",        "display_formula", "doc_title",     "figure_title",
    "footer",         "footer_image", "footnote",         "formula_number",
    "header",         "header_image", "image",            "inline_formula",
    "number",         "paragraph_title", "reference",     "reference_content",
    "seal",           "table",        "text",             "vertical_text",
    "vision_footnote",
};

Ort::Env& OrtEnv() {
  static Ort::Env env(ORT_LOGGING_LEVEL_WARNING, "trapo_vl_layout");
  return env;
}

#if defined(_WIN32)
std::wstring Utf8ToWide(const std::string& value) {
  if (value.empty()) {
    return std::wstring();
  }
  const int required = MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1,
                                           nullptr, 0);
  if (required <= 0) {
    throw std::runtime_error("failed to convert UTF-8 path to UTF-16");
  }
  std::wstring out(static_cast<size_t>(required - 1), L'\0');
  MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, out.data(), required);
  return out;
}
#endif

std::vector<float> MakeLayoutInput(const cv::Mat& bgr) {
  cv::Mat resized;
  cv::resize(bgr, resized, cv::Size(kLayoutInputSize, kLayoutInputSize), 0, 0,
             cv::INTER_LINEAR);
  cv::cvtColor(resized, resized, cv::COLOR_BGR2RGB);

  std::vector<float> out(3 * kLayoutInputSize * kLayoutInputSize);
  for (int y = 0; y < resized.rows; ++y) {
    const cv::Vec3b* row = resized.ptr<cv::Vec3b>(y);
    for (int x = 0; x < resized.cols; ++x) {
      const int offset = y * kLayoutInputSize + x;
      for (int c = 0; c < 3; ++c) {
        out[static_cast<size_t>(c * kLayoutInputSize * kLayoutInputSize +
                                offset)] =
            static_cast<float>(row[x][c]) / 255.0f;
      }
    }
  }
  return out;
}

cv::Rect2f ClampRect(float left, float top, float right, float bottom,
                     const cv::Size& size) {
  left = std::clamp(left, 0.0f, static_cast<float>(size.width));
  right = std::clamp(right, 0.0f, static_cast<float>(size.width));
  top = std::clamp(top, 0.0f, static_cast<float>(size.height));
  bottom = std::clamp(bottom, 0.0f, static_cast<float>(size.height));
  if (right < left) {
    std::swap(left, right);
  }
  if (bottom < top) {
    std::swap(top, bottom);
  }
  return cv::Rect2f(left, top, right - left, bottom - top);
}

bool LooksLikeClassScoreBox(const float* row, int columns) {
  return columns >= 6 && row[0] >= 0.0f &&
         row[0] < static_cast<float>(std::size(kLayoutLabels)) &&
         row[1] >= 0.0f && row[1] <= 1.01f;
}

}  // namespace

class LayoutAnalyzer::Impl {
 public:
  Impl(const std::string& model_path,
       const trapo_ocr_runtime_options_t& runtime) {
    Ort::SessionOptions options;
    options.SetGraphOptimizationLevel(GraphOptimizationLevel::ORT_ENABLE_ALL);
    if (runtime.cpu_threads > 0) {
      options.SetIntraOpNumThreads(runtime.cpu_threads);
    }

#if defined(_WIN32)
    const std::wstring wide_path = Utf8ToWide(model_path);
    session_ =
        std::make_unique<Ort::Session>(OrtEnv(), wide_path.c_str(), options);
#else
    session_ =
        std::make_unique<Ort::Session>(OrtEnv(), model_path.c_str(), options);
#endif

    Ort::AllocatorWithDefaultOptions allocator;
    for (size_t i = 0; i < session_->GetInputCount(); ++i) {
      auto name = session_->GetInputNameAllocated(i, allocator);
      input_names_.push_back(name.get());
    }
    for (size_t i = 0; i < session_->GetOutputCount(); ++i) {
      auto name = session_->GetOutputNameAllocated(i, allocator);
      output_names_.push_back(name.get());
    }
    for (const auto& name : input_names_) {
      input_name_ptrs_.push_back(name.c_str());
    }
    for (const auto& name : output_names_) {
      output_name_ptrs_.push_back(name.c_str());
    }
  }

  std::vector<LayoutRegion> Analyze(const cv::Mat& bgr) const {
    if (bgr.empty()) {
      throw std::invalid_argument("image is empty");
    }

    std::vector<float> image = MakeLayoutInput(bgr);
    std::vector<float> im_shape = {static_cast<float>(kLayoutInputSize),
                                   static_cast<float>(kLayoutInputSize)};
    std::vector<float> scale_factor = {
        static_cast<float>(kLayoutInputSize) / static_cast<float>(bgr.rows),
        static_cast<float>(kLayoutInputSize) / static_cast<float>(bgr.cols)};

    const std::vector<int64_t> image_shape = {1, 3, kLayoutInputSize,
                                              kLayoutInputSize};
    const std::vector<int64_t> aux_shape = {1, 2};
    Ort::MemoryInfo memory_info =
        Ort::MemoryInfo::CreateCpu(OrtArenaAllocator, OrtMemTypeDefault);
    std::vector<Ort::Value> inputs;
    inputs.reserve(input_names_.size());
    for (const std::string& name : input_names_) {
      if (name == "image") {
        inputs.push_back(Ort::Value::CreateTensor<float>(
            memory_info, image.data(), image.size(), image_shape.data(),
            image_shape.size()));
      } else if (name == "im_shape") {
        inputs.push_back(Ort::Value::CreateTensor<float>(
            memory_info, im_shape.data(), im_shape.size(), aux_shape.data(),
            aux_shape.size()));
      } else if (name == "scale_factor") {
        inputs.push_back(Ort::Value::CreateTensor<float>(
            memory_info, scale_factor.data(), scale_factor.size(),
            aux_shape.data(), aux_shape.size()));
      } else {
        throw std::runtime_error("unexpected PP-DocLayoutV3 input: " + name);
      }
    }

    auto outputs = session_->Run(Ort::RunOptions{nullptr},
                                 input_name_ptrs_.data(), inputs.data(),
                                 inputs.size(), output_name_ptrs_.data(),
                                 output_name_ptrs_.size());
    if (outputs.size() < 2 || !outputs[0].IsTensor() ||
        !outputs[1].IsTensor()) {
      throw std::runtime_error("PP-DocLayoutV3 returned unexpected outputs");
    }

    const auto bbox_info = outputs[0].GetTensorTypeAndShapeInfo();
    const std::vector<int64_t> bbox_shape = bbox_info.GetShape();
    if (bbox_shape.size() != 2 || bbox_shape[1] < 6) {
      throw std::runtime_error("PP-DocLayoutV3 detection shape is unsupported");
    }
    const int rows = static_cast<int>(bbox_shape[0]);
    const int columns = static_cast<int>(bbox_shape[1]);
    const float* data = outputs[0].GetTensorData<float>();
    const int32_t* counts = outputs[1].GetTensorData<int32_t>();
    const int count = std::clamp(counts[0], 0, rows);

    std::vector<LayoutRegion> regions;
    regions.reserve(static_cast<size_t>(count));
    for (int i = 0; i < count; ++i) {
      const float* row = data + static_cast<size_t>(i) * columns;
      int class_id = 0;
      float score = 0.0f;
      int box_offset = 2;
      if (LooksLikeClassScoreBox(row, columns)) {
        class_id = static_cast<int>(std::round(row[0]));
        score = row[1];
      } else {
        score = row[0];
        class_id = static_cast<int>(std::round(row[1]));
      }
      if (score < kLayoutThreshold || class_id < 0 ||
          class_id >= static_cast<int>(std::size(kLayoutLabels))) {
        continue;
      }

      cv::Rect2f box = ClampRect(row[box_offset], row[box_offset + 1],
                                 row[box_offset + 2], row[box_offset + 3],
                                 bgr.size());
      if (box.width < 2.0f || box.height < 2.0f) {
        continue;
      }

      LayoutRegion region;
      region.label = kLayoutLabels[class_id];
      region.confidence = score;
      region.bounding_box = box;
      region.polygon = {{box.x, box.y},
                        {box.x + box.width, box.y},
                        {box.x + box.width, box.y + box.height},
                        {box.x, box.y + box.height}};
      regions.push_back(std::move(region));
    }

    std::sort(regions.begin(), regions.end(),
              [](const LayoutRegion& a, const LayoutRegion& b) {
                const float ay = a.bounding_box.y;
                const float by = b.bounding_box.y;
                if (std::abs(ay - by) > 16.0f) {
                  return ay < by;
                }
                return a.bounding_box.x < b.bounding_box.x;
              });
    for (size_t i = 0; i < regions.size(); ++i) {
      regions[i].reading_order = static_cast<int>(i);
    }
    return regions;
  }

 private:
  std::unique_ptr<Ort::Session> session_;
  std::vector<std::string> input_names_;
  std::vector<std::string> output_names_;
  std::vector<const char*> input_name_ptrs_;
  std::vector<const char*> output_name_ptrs_;
};

LayoutAnalyzer::LayoutAnalyzer(const std::string& model_path,
                               const trapo_ocr_runtime_options_t& runtime)
    : impl_(std::make_shared<Impl>(model_path, runtime)) {}

std::vector<LayoutRegion> LayoutAnalyzer::Analyze(const cv::Mat& bgr) const {
  return impl_->Analyze(bgr);
}

}  // namespace trapo_ocr
