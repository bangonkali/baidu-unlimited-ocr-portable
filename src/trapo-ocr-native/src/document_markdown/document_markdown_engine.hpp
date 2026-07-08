#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_ENGINE_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_ENGINE_HPP_

#include <memory>
#include <random>
#include <string>
#include <vector>

#include "trapo_ocr.h"
#include "document_markdown/document_markdown_image_processor.hpp"
#include "document_markdown/document_markdown_onnx_runner.hpp"
#include "document_markdown/document_markdown_tokenizer.hpp"
#include "model/ocr_model_bundle.hpp"

namespace trapo_ocr {

class DocumentMarkdownEngine {
 public:
  DocumentMarkdownEngine(const DocumentMarkdownBundleCheck& bundle,
                      const trapo_ocr_runtime_options_t& runtime,
                      trapo_ocr_backend_t active_backend);
  ~DocumentMarkdownEngine();

  DocumentMarkdownEngine(const DocumentMarkdownEngine&) = delete;
  DocumentMarkdownEngine& operator=(const DocumentMarkdownEngine&) = delete;

  std::string Recognize(const trapo_ocr_image_t& image,
                        const trapo_ocr_run_options_t& options,
                        const std::vector<std::string>& warnings = {});

  std::string runtime_summary() const;
  trapo_ocr_backend_t active_backend() const { return active_backend_; }

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
  GenerationResult Generate(const DocumentMarkdownImageInputs& image,
                            const trapo_ocr_run_options_t& options);
  int64_t SelectNextToken(const std::vector<float>& logits, float temperature);
  std::string BuildResultJson(const DocumentMarkdownImageInputs& image,
                              const GenerationResult& generation,
                              int64_t total_ms,
                              const std::vector<std::string>& warnings) const;

  trapo_ocr_backend_t active_backend_ = TRAPO_OCR_BACKEND_CPU;
  trapo_ocr_backend_t vision_backend_ = TRAPO_OCR_BACKEND_CPU;
  int32_t cpu_threads_ = 0;
  bool enable_profiling_ = false;
  DocumentMarkdownTokenizer tokenizer_;
  DocumentMarkdownImageProcessor image_processor_;
  DocumentMarkdownOnnxSession vision_session_;
  DocumentMarkdownOnnxSession embed_session_;
  DocumentMarkdownOnnxSession decoder_session_;
  std::mt19937 random_;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_ENGINE_HPP_
