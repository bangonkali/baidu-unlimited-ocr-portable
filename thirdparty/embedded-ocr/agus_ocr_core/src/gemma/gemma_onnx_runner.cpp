#include "gemma/gemma_onnx_runner.hpp"

#include <algorithm>
#include <numeric>
#include <stdexcept>
#include <utility>

#include <onnxruntime_cxx_api.h>

#include "gemma/gemma_common.hpp"

#if defined(_WIN32)
#if defined(AGUS_OCR_ENABLE_DIRECTML)
#include <dml_provider_factory.h>
#endif
#endif

namespace agus_ocr {
namespace {

Ort::Env& GemmaOrtEnv() {
  static Ort::Env env(ORT_LOGGING_LEVEL_WARNING, "agus_docs_gemma");
  return env;
}

size_t ElementCount(const std::vector<int64_t>& shape) {
  if (shape.empty()) {
    return 1;
  }
  return static_cast<size_t>(std::accumulate(
      shape.begin(), shape.end(), int64_t{1}, std::multiplies<int64_t>()));
}

template <typename T>
Ort::Value CreateTensor(Ort::MemoryInfo& memory_info,
                        const std::vector<T>& data,
                        const std::vector<int64_t>& shape) {
  const size_t expected = ElementCount(shape);
  if (data.size() != expected) {
    throw std::runtime_error("Gemma ONNX input shape does not match data size");
  }
  static T dummy{};
  T* pointer = data.empty() ? &dummy : const_cast<T*>(data.data());
  const int64_t* dims = shape.empty() ? nullptr : shape.data();
  return Ort::Value::CreateTensor<T>(memory_info, pointer, data.size(), dims,
                                     shape.size());
}

}  // namespace

GemmaTensor GemmaTensor::Float(std::string name,
                               std::vector<int64_t> shape,
                               std::vector<float> data) {
  GemmaTensor tensor;
  tensor.name = std::move(name);
  tensor.type = GemmaTensorType::kFloat;
  tensor.shape = std::move(shape);
  tensor.floats = std::move(data);
  return tensor;
}

GemmaTensor GemmaTensor::Int64(std::string name,
                               std::vector<int64_t> shape,
                               std::vector<int64_t> data) {
  GemmaTensor tensor;
  tensor.name = std::move(name);
  tensor.type = GemmaTensorType::kInt64;
  tensor.shape = std::move(shape);
  tensor.ints = std::move(data);
  return tensor;
}

std::string GemmaBackendLabel(agus_ocr_backend_t backend) {
  switch (backend) {
    case AGUS_OCR_BACKEND_DIRECTML:
      return "DirectML/D3D12";
    case AGUS_OCR_BACKEND_CUDA:
      return "CUDA";
    case AGUS_OCR_BACKEND_CPU:
      return "CPU";
    default:
      return "CPU";
  }
}

GemmaOnnxSession::GemmaOnnxSession(const std::string& model_path,
                                   agus_ocr_backend_t backend,
                                   int32_t cpu_threads,
                                   bool enable_profiling,
                                   const std::string& session_name) {
  Ort::SessionOptions options;
  options.SetGraphOptimizationLevel(GraphOptimizationLevel::ORT_ENABLE_ALL);
  if (cpu_threads > 0) {
    options.SetIntraOpNumThreads(cpu_threads);
    options.SetInterOpNumThreads(1);
    options.SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
  }
  if (enable_profiling) {
#if defined(_WIN32)
    const std::wstring wide_session_name = GemmaUtf8ToWide(session_name);
    options.EnableProfiling(wide_session_name.c_str());
#else
    options.EnableProfiling(session_name.c_str());
#endif
  }

  if (backend == AGUS_OCR_BACKEND_DIRECTML) {
#if defined(_WIN32) && defined(AGUS_OCR_ENABLE_DIRECTML)
    options.DisableMemPattern();
    options.SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
    Ort::ThrowOnError(OrtSessionOptionsAppendExecutionProvider_DML(options, 0));
#else
    throw std::runtime_error("Gemma DirectML was selected in a non-DirectML build");
#endif
  }

  if (backend == AGUS_OCR_BACKEND_CUDA) {
#if defined(_WIN32)
    OrtCUDAProviderOptions cuda_options{};
    cuda_options.device_id = 0;
    const auto append_cuda =
        Ort::GetApi().SessionOptionsAppendExecutionProvider_CUDA;
    if (append_cuda == nullptr) {
      throw std::runtime_error(
          "Gemma CUDA selected but this ONNX Runtime build does not expose "
          "the CUDA provider API");
    }
    Ort::ThrowOnError(append_cuda(options, &cuda_options));
#else
    throw std::runtime_error("Gemma CUDA was selected in a non-Windows build");
#endif
  }

  GemmaLogInfo("core gemma create ONNX session name=" + session_name +
               " backend=" + GemmaBackendLabel(backend));
#if defined(_WIN32)
  const std::wstring wide_path = GemmaUtf8ToWide(model_path);
  session_ =
      std::make_unique<Ort::Session>(GemmaOrtEnv(), wide_path.c_str(), options);
#else
  session_ =
      std::make_unique<Ort::Session>(GemmaOrtEnv(), model_path.c_str(), options);
#endif

  Ort::AllocatorWithDefaultOptions allocator;
  const size_t input_count = session_->GetInputCount();
  input_infos_.reserve(input_count);
  input_names_.reserve(input_count);
  for (size_t i = 0; i < input_count; ++i) {
    auto name = session_->GetInputNameAllocated(i, allocator);
    GemmaInputInfo info;
    info.name = name.get();
    input_names_.push_back(info.name);
    auto type_info = session_->GetInputTypeInfo(i).GetTensorTypeAndShapeInfo();
    info.shape = type_info.GetShape();
    input_infos_.push_back(std::move(info));
  }

  const size_t output_count = session_->GetOutputCount();
  output_names_.reserve(output_count);
  output_name_ptrs_.reserve(output_count);
  for (size_t i = 0; i < output_count; ++i) {
    auto name = session_->GetOutputNameAllocated(i, allocator);
    output_names_.push_back(name.get());
  }
  for (const std::string& name : output_names_) {
    output_name_ptrs_.push_back(name.c_str());
  }
}

GemmaOnnxSession::~GemmaOnnxSession() = default;

std::vector<GemmaTensor> GemmaOnnxSession::Run(
    const std::vector<GemmaTensor>& inputs) const {
  Ort::MemoryInfo memory_info =
      Ort::MemoryInfo::CreateCpu(OrtArenaAllocator, OrtMemTypeDefault);
  std::vector<Ort::Value> values;
  std::vector<const char*> names;
  values.reserve(inputs.size());
  names.reserve(inputs.size());
  for (const GemmaTensor& input : inputs) {
    names.push_back(input.name.c_str());
    if (input.type == GemmaTensorType::kFloat) {
      values.push_back(CreateTensor(memory_info, input.floats, input.shape));
    } else {
      values.push_back(CreateTensor(memory_info, input.ints, input.shape));
    }
  }

  auto outputs = session_->Run(Ort::RunOptions{nullptr}, names.data(),
                               values.data(), values.size(),
                               output_name_ptrs_.data(),
                               output_name_ptrs_.size());
  std::vector<GemmaTensor> result;
  result.reserve(outputs.size());
  for (size_t i = 0; i < outputs.size(); ++i) {
    if (!outputs[i].IsTensor()) {
      throw std::runtime_error("Gemma ONNX output was not a tensor");
    }
    auto type_info = outputs[i].GetTensorTypeAndShapeInfo();
    const auto shape = type_info.GetShape();
    const size_t count = type_info.GetElementCount();
    GemmaTensor tensor;
    tensor.name = i < output_names_.size() ? output_names_[i] : "";
    tensor.shape = shape;
    const auto element_type = type_info.GetElementType();
    if (element_type == ONNX_TENSOR_ELEMENT_DATA_TYPE_INT64) {
      tensor.type = GemmaTensorType::kInt64;
      const int64_t* data = outputs[i].GetTensorData<int64_t>();
      tensor.ints.assign(data, data + count);
    } else {
      tensor.type = GemmaTensorType::kFloat;
      const float* data = outputs[i].GetTensorData<float>();
      tensor.floats.assign(data, data + count);
    }
    result.push_back(std::move(tensor));
  }
  return result;
}

}  // namespace agus_ocr
