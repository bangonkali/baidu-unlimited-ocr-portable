#ifndef AGUS_OCR_MODEL_OCR_MODEL_BUNDLE_HPP_
#define AGUS_OCR_MODEL_OCR_MODEL_BUNDLE_HPP_

#include <string>
#include <vector>

namespace agus_ocr {

struct PaddleOcrVlBundleCheck {
  bool ok = false;
  std::string root;
  std::string manifest_path;
  std::string layout_model_path;
  std::string vl_model_path;
  std::string vl_mmproj_path;
  std::vector<std::string> missing;

  std::string summary() const;
  std::string message() const;
};

struct GemmaMarkdownBundleCheck {
  bool ok = false;
  std::string root;
  std::string manifest_path;
  std::string doc_orientation_model_path;
  std::string config_path;
  std::string tokenizer_path;
  std::string vision_model_path;
  std::string embed_model_path;
  std::string decoder_model_path;
  std::vector<std::string> missing;

  std::string summary() const;
  std::string message() const;
};

std::string JoinPath(const std::string& root, const std::string& child);
bool FileExists(const std::string& path);

PaddleOcrVlBundleCheck ValidatePaddleOcrVl16Bundle(
    const std::string& root,
    const std::string& explicit_vl_model_path,
    const std::string& explicit_vl_mmproj_path);

GemmaMarkdownBundleCheck ValidateGemmaMarkdownBundle(const std::string& root);

}  // namespace agus_ocr

#endif  // AGUS_OCR_MODEL_OCR_MODEL_BUNDLE_HPP_
