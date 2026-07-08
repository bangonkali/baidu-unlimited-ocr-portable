#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_ONNX_RUNNER_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_ONNX_RUNNER_HPP_

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#include "trapo_ocr.h"

namespace Ort {
struct Session;
}

namespace trapo_ocr {

enum class DocumentMarkdownTensorType {
  kFloat,
  kInt64,
};

struct DocumentMarkdownTensor {
  std::string name;
  DocumentMarkdownTensorType type = DocumentMarkdownTensorType::kFloat;
  std::vector<int64_t> shape;
  std::vector<float> floats;
  std::vector<int64_t> ints;

  static DocumentMarkdownTensor Float(std::string name,
                           std::vector<int64_t> shape,
                           std::vector<float> data);
  static DocumentMarkdownTensor Int64(std::string name,
                           std::vector<int64_t> shape,
                           std::vector<int64_t> data);
};

struct DocumentMarkdownInputInfo {
  std::string name;
  std::vector<int64_t> shape;
};

class DocumentMarkdownOnnxSession {
 public:
  DocumentMarkdownOnnxSession(const std::string& model_path,
                   trapo_ocr_backend_t backend,
                   int32_t cpu_threads,
                   bool enable_profiling,
                   const std::string& session_name);
  ~DocumentMarkdownOnnxSession();

  DocumentMarkdownOnnxSession(const DocumentMarkdownOnnxSession&) = delete;
  DocumentMarkdownOnnxSession& operator=(const DocumentMarkdownOnnxSession&) = delete;

  std::vector<DocumentMarkdownTensor> Run(const std::vector<DocumentMarkdownTensor>& inputs) const;
  const std::vector<DocumentMarkdownInputInfo>& inputs() const { return input_infos_; }
  const std::vector<std::string>& output_names() const { return output_names_; }

 private:
  std::unique_ptr<Ort::Session> session_;
  std::vector<DocumentMarkdownInputInfo> input_infos_;
  std::vector<std::string> input_names_;
  std::vector<std::string> output_names_;
  std::vector<const char*> output_name_ptrs_;
};

std::string DocumentMarkdownBackendLabel(trapo_ocr_backend_t backend);

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_ONNX_RUNNER_HPP_
