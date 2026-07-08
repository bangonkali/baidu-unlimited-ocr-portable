#ifndef AGUS_OCR_GEMMA_MARKDOWN_ENGINE_HPP_
#define AGUS_OCR_GEMMA_MARKDOWN_ENGINE_HPP_

#include <memory>
#include <random>
#include <string>
#include <vector>

#include "agus_ocr.h"
#include "gemma/gemma_image_processor.hpp"
#include "gemma/gemma_onnx_runner.hpp"
#include "gemma/gemma_tokenizer.hpp"
#include "model/ocr_model_bundle.hpp"

namespace agus_ocr {

class GemmaMarkdownEngine {
 public:
  GemmaMarkdownEngine(const GemmaMarkdownBundleCheck& bundle,
                      const agus_ocr_runtime_options_t& runtime,
                      agus_ocr_backend_t active_backend);
  ~GemmaMarkdownEngine();

  GemmaMarkdownEngine(const GemmaMarkdownEngine&) = delete;
  GemmaMarkdownEngine& operator=(const GemmaMarkdownEngine&) = delete;

  std::string Recognize(const agus_ocr_image_t& image,
                        const agus_ocr_run_options_t& options,
                        const std::vector<std::string>& warnings = {});

  std::string runtime_summary() const;
  agus_ocr_backend_t active_backend() const { return active_backend_; }

 private:
  struct GenerationResult {
    std::string markdown;
    int32_t prompt_tokens = 0;
    int32_t vision_tokens = 0;
    int32_t generated_tokens = 0;
    int64_t vision_ms = 0;
    int64_t generation_ms = 0;
  };

  std::string BuildPrompt(int image_tokens,
                          const std::string& markdown_prompt) const;
  GenerationResult Generate(const GemmaImageInputs& image,
                            const agus_ocr_run_options_t& options);
  int64_t SelectNextToken(const std::vector<float>& logits, float temperature);
  std::string BuildResultJson(const GemmaImageInputs& image,
                              const GenerationResult& generation,
                              int64_t total_ms,
                              const std::vector<std::string>& warnings) const;

  agus_ocr_backend_t active_backend_ = AGUS_OCR_BACKEND_CPU;
  agus_ocr_backend_t vision_backend_ = AGUS_OCR_BACKEND_CPU;
  int32_t cpu_threads_ = 0;
  bool enable_profiling_ = false;
  GemmaTokenizer tokenizer_;
  GemmaImageProcessor image_processor_;
  GemmaOnnxSession vision_session_;
  GemmaOnnxSession embed_session_;
  GemmaOnnxSession decoder_session_;
  std::mt19937 random_;
};

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_MARKDOWN_ENGINE_HPP_
