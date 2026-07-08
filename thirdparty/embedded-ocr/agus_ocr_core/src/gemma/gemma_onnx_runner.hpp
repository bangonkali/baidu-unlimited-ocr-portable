#ifndef AGUS_OCR_GEMMA_ONNX_RUNNER_HPP_
#define AGUS_OCR_GEMMA_ONNX_RUNNER_HPP_

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#include "agus_ocr.h"

namespace Ort {
struct Session;
}

namespace agus_ocr {

enum class GemmaTensorType {
  kFloat,
  kInt64,
};

struct GemmaTensor {
  std::string name;
  GemmaTensorType type = GemmaTensorType::kFloat;
  std::vector<int64_t> shape;
  std::vector<float> floats;
  std::vector<int64_t> ints;

  static GemmaTensor Float(std::string name,
                           std::vector<int64_t> shape,
                           std::vector<float> data);
  static GemmaTensor Int64(std::string name,
                           std::vector<int64_t> shape,
                           std::vector<int64_t> data);
};

struct GemmaInputInfo {
  std::string name;
  std::vector<int64_t> shape;
};

class GemmaOnnxSession {
 public:
  GemmaOnnxSession(const std::string& model_path,
                   agus_ocr_backend_t backend,
                   int32_t cpu_threads,
                   bool enable_profiling,
                   const std::string& session_name);
  ~GemmaOnnxSession();

  GemmaOnnxSession(const GemmaOnnxSession&) = delete;
  GemmaOnnxSession& operator=(const GemmaOnnxSession&) = delete;

  std::vector<GemmaTensor> Run(const std::vector<GemmaTensor>& inputs) const;
  const std::vector<GemmaInputInfo>& inputs() const { return input_infos_; }
  const std::vector<std::string>& output_names() const { return output_names_; }

 private:
  std::unique_ptr<Ort::Session> session_;
  std::vector<GemmaInputInfo> input_infos_;
  std::vector<std::string> input_names_;
  std::vector<std::string> output_names_;
  std::vector<const char*> output_name_ptrs_;
};

std::string GemmaBackendLabel(agus_ocr_backend_t backend);

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_ONNX_RUNNER_HPP_
