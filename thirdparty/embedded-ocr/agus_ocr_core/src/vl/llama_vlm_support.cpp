#include "vl/llama_vlm_support.hpp"

#include <algorithm>
#include <cmath>
#include <thread>
#include <vector>

#include "opencv2/imgproc.hpp"

namespace agus_ocr {

int ResolveLlamaThreads(const agus_ocr_runtime_options_t& runtime) {
  if (runtime.cpu_threads > 0) {
    return runtime.cpu_threads;
  }
  const int hardware = static_cast<int>(std::thread::hardware_concurrency());
#if defined(__ANDROID__)
  return std::max(1, std::min(4, hardware <= 0 ? 4 : hardware));
#else
  return std::max(1, std::min(8, hardware <= 0 ? 4 : hardware));
#endif
}

cv::Mat ResizeToPixelBudget(const cv::Mat& bgr,
                            const VlGenerationOptions& options) {
  if (bgr.empty()) {
    return bgr;
  }
  const double area = static_cast<double>(bgr.cols) * bgr.rows;
  double scale = 1.0;
  if (options.max_pixels > 0 && area > options.max_pixels) {
    scale = std::sqrt(static_cast<double>(options.max_pixels) / area);
  } else if (options.min_pixels > 0 && area < options.min_pixels) {
    scale = std::sqrt(static_cast<double>(options.min_pixels) / area);
  }
  if (std::abs(scale - 1.0) < 0.01) {
    return bgr;
  }
  cv::Mat resized;
  cv::resize(bgr, resized,
             cv::Size(std::max(1, static_cast<int>(bgr.cols * scale)),
                      std::max(1, static_cast<int>(bgr.rows * scale))),
             0, 0, scale < 1.0 ? cv::INTER_AREA : cv::INTER_CUBIC);
  return resized;
}

std::string PromptForLayoutLabel(const std::string& label, bool markdown) {
  const std::string format =
      markdown ? "Return only Markdown." : "Return only plain text.";
  if (label == "table") {
    return "Convert this document table region to a GitHub-Flavored Markdown "
           "table. " +
           format;
  }
  if (label == "display_formula" || label == "inline_formula" ||
      label == "formula_number") {
    return "Transcribe this formula region using LaTeX-compatible Markdown. " +
           format;
  }
  if (label == "chart") {
    return "Extract the visible chart title, labels, and values as concise "
           "Markdown. " +
           format;
  }
  if (label == "seal") {
    return "Transcribe the visible seal or stamp text. " + format;
  }
  return "Transcribe this document region accurately. Preserve lists and "
         "line breaks when visible. " +
         format;
}

std::string TrimGeneratedText(std::string value) {
  const char* whitespace = " \t\r\n";
  const size_t begin = value.find_first_not_of(whitespace);
  if (begin == std::string::npos) {
    return "";
  }
  const size_t end = value.find_last_not_of(whitespace);
  value = value.substr(begin, end - begin + 1);
  const std::vector<std::string> stops = {"</s>", "<|end|>", "User:"};
  for (const std::string& stop : stops) {
    const size_t index = value.find(stop);
    if (index != std::string::npos) {
      value = value.substr(0, index);
    }
  }
  const size_t second_begin = value.find_first_not_of(whitespace);
  if (second_begin == std::string::npos) {
    return "";
  }
  const size_t second_end = value.find_last_not_of(whitespace);
  return value.substr(second_begin, second_end - second_begin + 1);
}

}  // namespace agus_ocr
