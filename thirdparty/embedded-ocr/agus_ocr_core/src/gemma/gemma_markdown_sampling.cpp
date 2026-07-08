#include "gemma/gemma_markdown_engine.hpp"

#include <algorithm>
#include <cmath>
#include <functional>
#include <random>
#include <stdexcept>

namespace agus_ocr {
namespace {

constexpr int32_t kTopK = 64;
constexpr double kTopP = 0.95;

}  // namespace

int64_t GemmaMarkdownEngine::SelectNextToken(
    const std::vector<float>& logits,
    float temperature) {
  if (logits.empty()) {
    throw std::runtime_error("Gemma decoder returned empty logits");
  }
  if (temperature <= 0.0f || !std::isfinite(temperature)) {
    return static_cast<int64_t>(std::distance(
        logits.begin(), std::max_element(logits.begin(), logits.end())));
  }

  std::vector<std::pair<float, int64_t>> top;
  top.reserve(kTopK);
  for (size_t i = 0; i < logits.size(); ++i) {
    const float value = logits[i];
    if (!std::isfinite(value)) {
      continue;
    }
    if (top.size() < kTopK) {
      top.push_back({value, static_cast<int64_t>(i)});
      if (top.size() == kTopK) {
        std::make_heap(top.begin(), top.end(), std::greater<>());
      }
    } else if (value > top.front().first) {
      std::pop_heap(top.begin(), top.end(), std::greater<>());
      top.back() = {value, static_cast<int64_t>(i)};
      std::push_heap(top.begin(), top.end(), std::greater<>());
    }
  }
  if (top.empty()) {
    return 0;
  }
  std::sort(top.begin(), top.end(),
            [](const auto& left, const auto& right) {
              return left.first > right.first;
            });

  const float max_logit = top.front().first;
  double total = 0.0;
  std::vector<double> weights;
  weights.reserve(top.size());
  for (const auto& item : top) {
    const double weight =
        std::exp((static_cast<double>(item.first) - max_logit) /
                 std::max(0.001f, temperature));
    weights.push_back(weight);
    total += weight;
  }

  double cumulative = 0.0;
  size_t keep = weights.size();
  for (size_t i = 0; i < weights.size(); ++i) {
    cumulative += weights[i] / total;
    if (cumulative >= kTopP) {
      keep = i + 1;
      break;
    }
  }
  std::discrete_distribution<size_t> distribution(weights.begin(),
                                                  weights.begin() + keep);
  return top[distribution(random_)].second;
}

}  // namespace agus_ocr
