#include "model/ocr_model_bundle.hpp"

#include <fstream>
#include <sstream>

namespace agus_ocr {
namespace {

bool IsAbsolutePath(const std::string& path) {
  if (path.empty()) {
    return false;
  }
#if defined(_WIN32)
  return path.size() >= 3 && path[1] == ':' &&
         (path[2] == '\\' || path[2] == '/');
#else
  return path[0] == '/';
#endif
}

std::string FirstExistingPath(const std::vector<std::string>& candidates) {
  for (const std::string& candidate : candidates) {
    if (FileExists(candidate)) {
      return candidate;
    }
  }
  return candidates.empty() ? std::string() : candidates.front();
}

}  // namespace

std::string JoinPath(const std::string& root, const std::string& child) {
  if (root.empty() || IsAbsolutePath(child)) {
    return child;
  }
  const char last = root[root.size() - 1];
  if (last == '/' || last == '\\') {
    return root + child;
  }
  return root + "/" + child;
}

bool FileExists(const std::string& path) {
  if (path.empty()) {
    return false;
  }
  std::ifstream input(path, std::ios::binary);
  return input.good();
}

std::string PaddleOcrVlBundleCheck::summary() const {
  std::ostringstream out;
  out << "PaddleOCR-VL-1.6 native bundle";
  if (!root.empty()) {
    out << " root=" << root;
  }
  return out.str();
}

std::string PaddleOcrVlBundleCheck::message() const {
  if (ok) {
    return "PaddleOCR-VL-1.6 bundle is present.";
  }
  std::ostringstream out;
  out << "PaddleOCR-VL-1.6 model bundle is incomplete.";
  if (!missing.empty()) {
    out << " Missing:";
    for (const std::string& path : missing) {
      out << " " << path;
    }
  }
  return out.str();
}

std::string GemmaMarkdownBundleCheck::summary() const {
  std::ostringstream out;
  out << "Gemma Markdown ONNX bundle";
  if (!root.empty()) {
    out << " root=" << root;
  }
  return out.str();
}

std::string GemmaMarkdownBundleCheck::message() const {
  if (ok) {
    return "Gemma Markdown bundle is present.";
  }
  std::ostringstream out;
  out << "Gemma Markdown model bundle is incomplete.";
  if (!missing.empty()) {
    out << " Missing:";
    for (const std::string& path : missing) {
      out << " " << path;
    }
  }
  return out.str();
}

PaddleOcrVlBundleCheck ValidatePaddleOcrVl16Bundle(
    const std::string& root,
    const std::string& explicit_vl_model_path,
    const std::string& explicit_vl_mmproj_path) {
  PaddleOcrVlBundleCheck check;
  check.root = root;
  check.manifest_path = JoinPath(root, "manifest.json");
  check.layout_model_path = FirstExistingPath({
      JoinPath(root, "layout_detection/inference.onnx"),
      JoinPath(root, "layout_detection/PP-DocLayoutV3.onnx"),
  });
  check.vl_model_path =
      explicit_vl_model_path.empty()
          ? FirstExistingPath({
                JoinPath(root, "vl/PaddleOCR-VL-1.6-GGUF.gguf"),
                JoinPath(root, "PaddleOCR-VL-1.6-GGUF.gguf"),
                JoinPath(root, "PaddleOCR-VL-1.6.gguf"),
            })
          : JoinPath(root, explicit_vl_model_path);
  check.vl_mmproj_path =
      explicit_vl_mmproj_path.empty()
          ? FirstExistingPath({
                JoinPath(root, "vl/PaddleOCR-VL-1.6-GGUF-mmproj.gguf"),
                JoinPath(root, "PaddleOCR-VL-1.6-GGUF-mmproj.gguf"),
                JoinPath(root, "PaddleOCR-VL-1.6-mmproj.gguf"),
            })
          : JoinPath(root, explicit_vl_mmproj_path);

  if (!FileExists(check.manifest_path)) {
    check.missing.push_back(check.manifest_path);
  }
  if (!FileExists(check.layout_model_path)) {
    check.missing.push_back(check.layout_model_path);
  }
  if (!FileExists(check.vl_model_path)) {
    check.missing.push_back(check.vl_model_path);
  }
  if (!FileExists(check.vl_mmproj_path)) {
    check.missing.push_back(check.vl_mmproj_path);
  }
  check.ok = check.missing.empty();
  return check;
}

GemmaMarkdownBundleCheck ValidateGemmaMarkdownBundle(const std::string& root) {
  GemmaMarkdownBundleCheck check;
  check.root = root;
  check.manifest_path = JoinPath(root, "manifest.json");
  check.doc_orientation_model_path =
      JoinPath(root, "doc_orientation/inference.onnx");
  check.config_path = JoinPath(root, "config/config.json");
  check.tokenizer_path = JoinPath(root, "config/tokenizer.json");
  check.vision_model_path = JoinPath(root, "onnx/vision_encoder_q4.onnx");
  check.embed_model_path = JoinPath(root, "onnx/embed_tokens_q4.onnx");
  check.decoder_model_path =
      JoinPath(root, "onnx/decoder_model_merged_q4.onnx");

  for (const std::string& path :
       {check.manifest_path, check.doc_orientation_model_path,
        check.config_path, check.tokenizer_path, check.vision_model_path,
        check.embed_model_path, check.decoder_model_path}) {
    if (!FileExists(path)) {
      check.missing.push_back(path);
    }
  }
  for (const std::string& path :
       {JoinPath(root, "onnx/vision_encoder_q4.onnx_data"),
        JoinPath(root, "onnx/embed_tokens_q4.onnx_data"),
        JoinPath(root, "onnx/embed_tokens_q4.onnx_data_1"),
        JoinPath(root, "onnx/decoder_model_merged_q4.onnx_data"),
        JoinPath(root, "onnx/decoder_model_merged_q4.onnx_data_1")}) {
    if (!FileExists(path)) {
      check.missing.push_back(path);
    }
  }
  check.ok = check.missing.empty();
  return check;
}

}  // namespace agus_ocr
