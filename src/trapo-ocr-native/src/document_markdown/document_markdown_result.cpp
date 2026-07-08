#include "document_markdown/document_markdown_engine.hpp"

#include <chrono>
#include <sstream>

#include "document_markdown/document_markdown_common.hpp"

namespace trapo_ocr {
namespace {

void AppendStringArray(std::ostringstream* out,
                       const std::vector<std::string>& values) {
  *out << '[';
  for (size_t i = 0; i < values.size(); ++i) {
    if (i > 0) {
      *out << ',';
    }
    *out << '"' << DocumentMarkdownJsonEscape(values[i]) << '"';
  }
  *out << ']';
}

}  // namespace

std::string DocumentMarkdownEngine::BuildResultJson(
    const DocumentMarkdownImageInputs& image,
    const GenerationResult& generation,
    int64_t total_ms,
    const std::vector<std::string>& warnings) const {
  std::ostringstream structured;
  structured << "{\"engine\":\"documentMarkdown\",\"backend\":\""
             << DocumentMarkdownJsonEscape(DocumentMarkdownBackendLabel(active_backend_))
             << "\",\"visionBackend\":\""
             << DocumentMarkdownJsonEscape(DocumentMarkdownBackendLabel(vision_backend_))
             << "\",\"softTokens\":" << image.soft_tokens
             << ",\"visionTokens\":" << generation.vision_tokens
             << ",\"maxPatches\":" << image.max_patches
             << ",\"promptTokens\":" << generation.prompt_tokens
             << ",\"generatedTokens\":" << generation.generated_tokens
             << ",\"visionMs\":" << generation.vision_ms
             << ",\"generationMs\":" << generation.generation_ms << "}";
  const std::string structured_json = structured.str();

  std::ostringstream out;
  out << "{\"status\":0,\"message\":\"ok\",\"pages\":[{\"pageIndex\":0,"
      << "\"width\":" << image.oriented_page.cols
      << ",\"height\":" << image.oriented_page.rows
      << ",\"overlayImageBytesBase64\":\"" << image.overlay_image_base64
      << "\",\"overlayImageMimeType\":\"" << image.overlay_mime_type
      << "\",\"docAngle\":" << image.doc_angle
      << ",\"lines\":[],\"text\":\"" << DocumentMarkdownJsonEscape(generation.markdown)
      << "\",\"markdownText\":\"" << DocumentMarkdownJsonEscape(generation.markdown)
      << "\",\"structuredJson\":\"" << DocumentMarkdownJsonEscape(structured_json)
      << "\",\"blocks\":[],\"annotationLayers\":[]}],\"text\":\""
      << DocumentMarkdownJsonEscape(generation.markdown) << "\",\"markdownText\":\""
      << DocumentMarkdownJsonEscape(generation.markdown) << "\",\"structuredJson\":\""
      << DocumentMarkdownJsonEscape(structured_json)
      << "\",\"timing\":{\"docOrientationMs\":0,\"docUnwarpingMs\":0,"
      << "\"detectionMs\":" << generation.vision_ms
      << ",\"textLineOrientationMs\":0,\"recognitionMs\":"
      << generation.generation_ms << ",\"totalMs\":" << total_ms
      << "},\"modelSummary\":\"Document Markdown ONNX q4 image-to-text\","
      << "\"runtimeSummary\":\"" << DocumentMarkdownJsonEscape(runtime_summary())
      << "\",\"warnings\":";
  AppendStringArray(&out, warnings);
  out << "}";
  return out.str();
}

std::string DocumentMarkdownEngine::Recognize(
    const trapo_ocr_image_t& image,
    const trapo_ocr_run_options_t& options,
    const std::vector<std::string>& warnings) {
  const auto total_start = std::chrono::steady_clock::now();
  DocumentMarkdownLogInfo("core document markdown recognize start bytes=" +
               std::to_string(image.length) + " backend=" +
               DocumentMarkdownBackendLabel(active_backend_));
  const int32_t visual_budget =
      options.visual_token_budget > 0 ? options.visual_token_budget : 280;
  DocumentMarkdownImageInputs inputs = image_processor_.Process(
      image, visual_budget, options.use_doc_orientation != 0);
  GenerationResult generation = Generate(inputs, options);
  const int64_t total_ms =
      DocumentMarkdownElapsedMs(total_start, std::chrono::steady_clock::now());
  DocumentMarkdownLogInfo("core document markdown recognize complete generatedTokens=" +
               std::to_string(generation.generated_tokens) +
               " elapsedMs=" + std::to_string(total_ms));
  return BuildResultJson(inputs, generation, total_ms, warnings);
}

}  // namespace trapo_ocr
