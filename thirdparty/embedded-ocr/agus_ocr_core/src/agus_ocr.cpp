#include "agus_ocr.h"

#include <algorithm>
#include <cctype>
#include <chrono>
#include <cmath>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <limits>
#include <memory>
#include <mutex>
#include <numeric>
#include <random>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

#include "model/ocr_model_bundle.hpp"

#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
#include <onnxruntime_cxx_api.h>
#include <opencv2/core.hpp>
#if defined(AGUS_OCR_USE_OPENCV_MOBILE) && __has_include(<opencv2/highgui/highgui.hpp>)
#include <opencv2/highgui/highgui.hpp>
#else
#include <opencv2/imgcodecs.hpp>
#endif
#include <opencv2/features2d.hpp>
#include <opencv2/imgproc.hpp>

#include "ocr_clipper.hpp"
#include "gemma/gemma_markdown_engine.hpp"
#include "vl/llama_vlm_runner.hpp"
#include "vl/paddle_ocr_vl_engine.hpp"

#if defined(__ANDROID__)
#include <android/api-level.h>
#include <android/log.h>
#include <sys/system_properties.h>
#if __has_include(<onnxruntime/core/providers/nnapi/nnapi_provider_factory.h>)
#include <onnxruntime/core/providers/nnapi/nnapi_provider_factory.h>
#define AGUS_OCR_HAS_NNAPI_PROVIDER_FACTORY 1
#elif __has_include(<nnapi_provider_factory.h>)
#include <nnapi_provider_factory.h>
#define AGUS_OCR_HAS_NNAPI_PROVIDER_FACTORY 1
#endif
#endif

#if defined(_WIN32)
#include <windows.h>
#if defined(AGUS_OCR_ENABLE_DIRECTML)
#include <d3d12.h>
#include <dxgi1_2.h>
#include <dml_provider_factory.h>
#endif
#endif
#endif

namespace {

thread_local std::string g_last_error;

constexpr const char* kModelSummary = "PP-OCRv6 medium full pipeline";
constexpr int kAndroidDefaultCpuThreads = 2;
constexpr int64_t kMaxDetectionPixels = 2000000;

struct RunOptions {
  bool use_doc_orientation = true;
  bool use_doc_unwarping = false;
  bool use_textline_orientation = true;
  int text_detection_limit_side_len = 736;
  std::string text_detection_limit_type = "min";
  float text_detection_threshold = 0.2f;
  float text_detection_box_threshold = 0.45f;
  float text_detection_unclip_ratio = 1.4f;
  float text_recognition_score_threshold = 0.0f;
  bool enable_source_box_estimation = true;
  bool generate_markdown = false;
  int32_t max_new_tokens = 1024;
  float temperature = 0.0f;
  int32_t min_pixels = 0;
  int32_t max_pixels = 2500000;
  std::string markdown_prompt =
      "Convert the document image to clean Markdown. Preserve headings, "
      "paragraphs, lists, tables, formulas, and reading order. Output only "
      "Markdown. Do not describe the image.";
  int32_t visual_token_budget = 560;
};

struct RuntimeOptions {
  agus_ocr_backend_t backend = AGUS_OCR_BACKEND_AUTO;
  int32_t cpu_threads = 0;
  bool enable_ort_profiling = false;
  bool force_cpu_only = false;
  agus_ocr_generative_backend_t generative_backend =
      AGUS_OCR_GEN_BACKEND_AUTO;
  int32_t generative_gpu_layers = 0;
};

struct AcceleratorStatus {
  agus_ocr_backend_t backend = AGUS_OCR_BACKEND_CPU;
  bool supported = false;
  bool enabled_by_default = false;
  std::string device_name;
  std::string unavailable_reason;
  std::string last_failure;
};

struct GenerativeAcceleratorStatus {
  agus_ocr_generative_backend_t backend = AGUS_OCR_GEN_BACKEND_CPU;
  bool supported = false;
  bool enabled_by_default = false;
  std::string device_name;
  std::string unavailable_reason;
  std::string last_failure;
};

struct Timing {
  int64_t doc_orientation_ms = 0;
  int64_t doc_unwarping_ms = 0;
  int64_t detection_ms = 0;
  int64_t textline_orientation_ms = 0;
  int64_t recognition_ms = 0;
  int64_t total_ms = 0;
};

agus_ocr_status_t fail(agus_ocr_status_t status, const std::string& message) {
  g_last_error = message;
  return status;
}

bool has_size(size_t actual, size_t expected) { return actual >= expected; }

constexpr size_t kRequiredRunOptionsSize =
    offsetof(agus_ocr_run_options_t, enable_source_box_estimation);
constexpr size_t kRequiredRuntimeOptionsSize =
    offsetof(agus_ocr_runtime_options_t, force_cpu_only);

RunOptions run_options_from_c(const agus_ocr_run_options_t& options) {
  RunOptions out;
  out.use_doc_orientation = options.use_doc_orientation != 0;
  out.use_doc_unwarping = options.use_doc_unwarping != 0;
  out.use_textline_orientation = options.use_textline_orientation != 0;
  out.text_detection_limit_side_len =
      options.text_detection_limit_side_len > 0
          ? options.text_detection_limit_side_len
          : out.text_detection_limit_side_len;
  if (options.text_detection_limit_type != nullptr &&
      options.text_detection_limit_type[0] != '\0') {
    out.text_detection_limit_type = options.text_detection_limit_type;
  }
  out.text_detection_threshold = options.text_detection_threshold;
  out.text_detection_box_threshold = options.text_detection_box_threshold;
  out.text_detection_unclip_ratio = options.text_detection_unclip_ratio;
  out.text_recognition_score_threshold =
      options.text_recognition_score_threshold;
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t,
                        enable_source_box_estimation) +
                   sizeof(options.enable_source_box_estimation))) {
    out.enable_source_box_estimation =
        options.enable_source_box_estimation != 0;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, generate_markdown) +
                   sizeof(options.generate_markdown))) {
    out.generate_markdown = options.generate_markdown != 0;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, max_new_tokens) +
                   sizeof(options.max_new_tokens))) {
    out.max_new_tokens =
        options.max_new_tokens > 0 ? options.max_new_tokens : out.max_new_tokens;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, temperature) +
                   sizeof(options.temperature))) {
    out.temperature = options.temperature;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, min_pixels) +
                   sizeof(options.min_pixels))) {
    out.min_pixels = options.min_pixels;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, max_pixels) +
                   sizeof(options.max_pixels))) {
    out.max_pixels =
        options.max_pixels > 0 ? options.max_pixels : out.max_pixels;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, markdown_prompt) +
                   sizeof(options.markdown_prompt)) &&
      options.markdown_prompt != nullptr &&
      options.markdown_prompt[0] != '\0') {
    out.markdown_prompt = options.markdown_prompt;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_run_options_t, visual_token_budget) +
                   sizeof(options.visual_token_budget)) &&
      options.visual_token_budget > 0) {
    out.visual_token_budget = options.visual_token_budget;
  }
  return out;
}

agus_ocr_run_options_t run_options_to_c(const RunOptions& options) {
  agus_ocr_run_options_t out{};
  out.struct_size = sizeof(out);
  out.use_doc_orientation = options.use_doc_orientation ? 1 : 0;
  out.use_doc_unwarping = options.use_doc_unwarping ? 1 : 0;
  out.use_textline_orientation = options.use_textline_orientation ? 1 : 0;
  out.text_detection_limit_side_len = options.text_detection_limit_side_len;
  out.text_detection_limit_type = options.text_detection_limit_type.c_str();
  out.text_detection_threshold = options.text_detection_threshold;
  out.text_detection_box_threshold = options.text_detection_box_threshold;
  out.text_detection_unclip_ratio = options.text_detection_unclip_ratio;
  out.text_recognition_score_threshold =
      options.text_recognition_score_threshold;
  out.enable_source_box_estimation =
      options.enable_source_box_estimation ? 1 : 0;
  out.generate_markdown = options.generate_markdown ? 1 : 0;
  out.max_new_tokens = options.max_new_tokens;
  out.temperature = options.temperature;
  out.min_pixels = options.min_pixels;
  out.max_pixels = options.max_pixels;
  out.markdown_prompt = options.markdown_prompt.c_str();
  out.visual_token_budget = options.visual_token_budget;
  return out;
}

RuntimeOptions runtime_options_from_c(
    const agus_ocr_runtime_options_t& options) {
  RuntimeOptions out;
  out.backend = options.backend;
  out.cpu_threads = options.cpu_threads;
  out.enable_ort_profiling = options.enable_ort_profiling != 0;
  if (has_size(options.struct_size,
               offsetof(agus_ocr_runtime_options_t, force_cpu_only))) {
    out.generative_backend = options.generative_backend;
    out.generative_gpu_layers = options.generative_gpu_layers;
  }
  if (has_size(options.struct_size,
               offsetof(agus_ocr_runtime_options_t, force_cpu_only) +
                   sizeof(options.force_cpu_only))) {
    out.force_cpu_only = options.force_cpu_only != 0;
  }
  if (out.force_cpu_only) {
    out.backend = AGUS_OCR_BACKEND_CPU;
    out.generative_backend = AGUS_OCR_GEN_BACKEND_CPU;
    out.generative_gpu_layers = 0;
  }
  return out;
}

agus_ocr_runtime_options_t runtime_options_to_c(const RuntimeOptions& options) {
  agus_ocr_runtime_options_t out{};
  out.struct_size = sizeof(out);
  out.backend = options.backend;
  out.cpu_threads = options.cpu_threads;
  out.enable_ort_profiling = options.enable_ort_profiling ? 1 : 0;
  out.generative_backend = options.generative_backend;
  out.generative_gpu_layers = options.generative_gpu_layers;
  out.force_cpu_only = options.force_cpu_only ? 1 : 0;
  return out;
}

const char* platform_label() {
#if defined(__ANDROID__)
  return "android";
#elif defined(_WIN32)
  return "windows";
#elif defined(__APPLE__)
  return "apple";
#elif defined(__linux__)
  return "linux";
#else
  return "native";
#endif
}

void core_log_info(const std::string& message) {
#if defined(__ANDROID__)
  __android_log_print(ANDROID_LOG_INFO, "AgusDocsOCR", "%s", message.c_str());
#else
  const char* enabled = std::getenv("AGUS_OCR_LOG");
  if (enabled != nullptr && enabled[0] != '\0' && enabled[0] != '0') {
    std::fprintf(stderr, "%s\n", message.c_str());
    std::fflush(stderr);
  }
#endif
}

int effective_cpu_threads(const RuntimeOptions& runtime) {
  if (runtime.cpu_threads > 0) {
    return runtime.cpu_threads;
  }
#if defined(__ANDROID__)
  return kAndroidDefaultCpuThreads;
#else
  return 0;
#endif
}

std::mutex& backend_health_mutex() {
  static std::mutex mutex;
  return mutex;
}

std::string& backend_last_failure(agus_ocr_backend_t backend) {
  static std::string directml_failure;
  static std::string xnnpack_failure;
  static std::string nnapi_failure;
  static std::string qnn_failure;
  switch (backend) {
    case AGUS_OCR_BACKEND_DIRECTML:
      return directml_failure;
    case AGUS_OCR_BACKEND_XNNPACK:
      return xnnpack_failure;
    case AGUS_OCR_BACKEND_NNAPI:
      return nnapi_failure;
    case AGUS_OCR_BACKEND_QNN:
      return qnn_failure;
    default:
      return directml_failure;
  }
}

bool backend_is_unhealthy(agus_ocr_backend_t backend,
                          std::string* reason = nullptr) {
  std::lock_guard<std::mutex> lock(backend_health_mutex());
  const std::string& failure = backend_last_failure(backend);
  const bool unhealthy = !failure.empty();
  if (reason != nullptr) {
    *reason = failure;
  }
  return unhealthy;
}

void mark_backend_unhealthy(agus_ocr_backend_t backend,
                            const std::string& reason) {
  std::lock_guard<std::mutex> lock(backend_health_mutex());
  backend_last_failure(backend) = reason;
}

bool directml_is_unhealthy(std::string* reason = nullptr) {
  return backend_is_unhealthy(AGUS_OCR_BACKEND_DIRECTML, reason);
}

void mark_directml_unhealthy(const std::string& reason) {
  mark_backend_unhealthy(AGUS_OCR_BACKEND_DIRECTML, reason);
}

std::string& gemma_directml_last_failure() {
  static std::string failure;
  return failure;
}

std::string& gemma_cuda_last_failure() {
  static std::string failure;
  return failure;
}

std::string& gemma_backend_last_failure(agus_ocr_backend_t backend) {
  return backend == AGUS_OCR_BACKEND_CUDA ? gemma_cuda_last_failure()
                                          : gemma_directml_last_failure();
}

bool gemma_backend_is_unhealthy(agus_ocr_backend_t backend,
                                std::string* reason = nullptr) {
  if (backend != AGUS_OCR_BACKEND_DIRECTML &&
      backend != AGUS_OCR_BACKEND_CUDA) {
    if (reason != nullptr) {
      reason->clear();
    }
    return false;
  }
  std::lock_guard<std::mutex> lock(backend_health_mutex());
  const std::string& failure = gemma_backend_last_failure(backend);
  const bool unhealthy = !failure.empty();
  if (reason != nullptr) {
    *reason = failure;
  }
  return unhealthy;
}

void mark_gemma_backend_unhealthy(agus_ocr_backend_t backend,
                                  const std::string& reason) {
  if (backend != AGUS_OCR_BACKEND_DIRECTML &&
      backend != AGUS_OCR_BACKEND_CUDA) {
    return;
  }
  std::lock_guard<std::mutex> lock(backend_health_mutex());
  gemma_backend_last_failure(backend) = reason;
}

const char* backend_label(agus_ocr_backend_t backend) {
  switch (backend) {
    case AGUS_OCR_BACKEND_AUTO:
      return "auto";
    case AGUS_OCR_BACKEND_CPU:
      return "cpu";
    case AGUS_OCR_BACKEND_XNNPACK:
      return "xnnpack";
    case AGUS_OCR_BACKEND_COREML:
      return "coreml";
    case AGUS_OCR_BACKEND_DIRECTML:
      return "directml";
    case AGUS_OCR_BACKEND_WEBASSEMBLY:
      return "webassembly";
    case AGUS_OCR_BACKEND_WEBGPU:
      return "webgpu";
    case AGUS_OCR_BACKEND_NNAPI:
      return "nnapi";
    case AGUS_OCR_BACKEND_QNN:
      return "qnn";
    case AGUS_OCR_BACKEND_CUDA:
      return "cuda";
  }
  return "unknown";
}

const char* generative_backend_label(agus_ocr_generative_backend_t backend) {
  switch (backend) {
    case AGUS_OCR_GEN_BACKEND_AUTO:
      return "auto";
    case AGUS_OCR_GEN_BACKEND_CPU:
      return "cpu";
    case AGUS_OCR_GEN_BACKEND_VULKAN:
      return "vulkan";
    case AGUS_OCR_GEN_BACKEND_CUDA:
      return "cuda";
    case AGUS_OCR_GEN_BACKEND_OPENCL:
      return "opencl";
  }
  return "unknown";
}

struct RuntimeCapabilities {
  std::string platform = platform_label();
  agus_ocr_backend_t default_backend = AGUS_OCR_BACKEND_CPU;
  bool directml_supported = false;
  bool directml_enabled_by_default = false;
  std::string directml_device_name;
  std::string directml_unavailable_reason = "DirectML is not available on this platform.";
  std::vector<AcceleratorStatus> accelerators;
  std::vector<GenerativeAcceleratorStatus> generative_accelerators;
  std::string runtime_summary;
};

std::string json_escape(const std::string& value);

std::string make_runtime_summary(agus_ocr_backend_t backend,
                                 int32_t cpu_threads) {
  std::ostringstream out;
  out << platform_label() << "-cpp onnxruntime " << backend_label(backend)
      << " opencv";
  if (cpu_threads > 0) {
    out << " threads=" << cpu_threads;
  }
  return out.str();
}

void append_accelerator_json(std::ostringstream* out,
                             const AcceleratorStatus& accelerator) {
  *out << "{\"backend\":" << static_cast<int>(accelerator.backend)
       << ",\"supported\":" << (accelerator.supported ? "true" : "false")
       << ",\"enabledByDefault\":"
       << (accelerator.enabled_by_default ? "true" : "false")
       << ",\"deviceName\":\"" << json_escape(accelerator.device_name)
       << "\",\"unavailableReason\":\""
       << json_escape(accelerator.unavailable_reason)
       << "\",\"lastFailure\":\"" << json_escape(accelerator.last_failure)
       << "\"}";
}

void append_generative_accelerator_json(
    std::ostringstream* out,
    const GenerativeAcceleratorStatus& accelerator) {
  *out << "{\"backend\":" << static_cast<int>(accelerator.backend)
       << ",\"supported\":" << (accelerator.supported ? "true" : "false")
       << ",\"enabledByDefault\":"
       << (accelerator.enabled_by_default ? "true" : "false")
       << ",\"deviceName\":\"" << json_escape(accelerator.device_name)
       << "\",\"unavailableReason\":\""
       << json_escape(accelerator.unavailable_reason)
       << "\",\"lastFailure\":\"" << json_escape(accelerator.last_failure)
       << "\"}";
}

std::string capabilities_json(const RuntimeCapabilities& capabilities) {
  std::ostringstream out;
  out << "{\"platform\":\"" << json_escape(capabilities.platform)
      << "\",\"defaultBackend\":"
      << static_cast<int>(capabilities.default_backend)
      << ",\"directMlSupported\":"
      << (capabilities.directml_supported ? "true" : "false")
      << ",\"directMlEnabledByDefault\":"
      << (capabilities.directml_enabled_by_default ? "true" : "false")
      << ",\"directMlDeviceName\":\""
      << json_escape(capabilities.directml_device_name)
      << "\",\"directMlUnavailableReason\":\""
      << json_escape(capabilities.directml_unavailable_reason)
      << "\",\"accelerators\":[";
  for (size_t i = 0; i < capabilities.accelerators.size(); ++i) {
    if (i > 0) {
      out << ',';
    }
    append_accelerator_json(&out, capabilities.accelerators[i]);
  }
  out << "],\"generativeAccelerators\":[";
  for (size_t i = 0; i < capabilities.generative_accelerators.size(); ++i) {
    if (i > 0) {
      out << ',';
    }
    append_generative_accelerator_json(
        &out, capabilities.generative_accelerators[i]);
  }
  out << "],\"runtimeSummary\":\""
      << json_escape(capabilities.runtime_summary) << "\"}";
  return out.str();
}

std::string join_path(const std::string& root, const std::string& child) {
  if (root.empty()) {
    return child;
  }
  const char last = root[root.size() - 1];
  if (last == '/' || last == '\\') {
    return root + child;
  }
  return root + "/" + child;
}

std::string json_escape(const std::string& value) {
  std::ostringstream out;
  for (unsigned char c : value) {
    switch (c) {
      case '"':
        out << "\\\"";
        break;
      case '\\':
        out << "\\\\";
        break;
      case '\b':
        out << "\\b";
        break;
      case '\f':
        out << "\\f";
        break;
      case '\n':
        out << "\\n";
        break;
      case '\r':
        out << "\\r";
        break;
      case '\t':
        out << "\\t";
        break;
      default:
        if (c < 0x20) {
          out << "\\u" << std::hex << std::setw(4) << std::setfill('0')
              << static_cast<int>(c) << std::dec << std::setfill(' ');
        } else {
          out << static_cast<char>(c);
        }
        break;
    }
  }
  return out.str();
}

std::string make_error_json(agus_ocr_status_t status, const std::string& message,
                            const std::string& runtime_summary) {
  std::ostringstream out;
  out << "{\"status\":" << static_cast<int>(status) << ",\"message\":\""
      << json_escape(message)
      << "\",\"pages\":[],\"text\":\"\",\"timing\":{\"docOrientationMs\":0,"
         "\"docUnwarpingMs\":0,\"detectionMs\":0,"
         "\"textLineOrientationMs\":0,\"recognitionMs\":0,\"totalMs\":0},"
      << "\"modelSummary\":\"" << kModelSummary << "\",\"runtimeSummary\":\""
      << json_escape(runtime_summary)
      << "\",\"markdownText\":\"\",\"structuredJson\":\"{}\",\"warnings\":[\""
      << json_escape(message) << "\"]}";
  return out.str();
}

agus_ocr_result_t* allocate_result(const std::string& json) {
  auto* result = static_cast<agus_ocr_result_t*>(
      std::calloc(1, sizeof(agus_ocr_result_t)));
  if (result == nullptr) {
    throw std::bad_alloc();
  }
  result->struct_size = sizeof(agus_ocr_result_t);
  result->json_length = json.size();
  result->json = static_cast<char*>(std::malloc(json.size() + 1));
  if (result->json == nullptr) {
    std::free(result);
    throw std::bad_alloc();
  }
  std::memcpy(result->json, json.data(), json.size());
  result->json[json.size()] = '\0';
  return result;
}

int64_t elapsed_ms(std::chrono::steady_clock::time_point start,
                   std::chrono::steady_clock::time_point end) {
  return std::chrono::duration_cast<std::chrono::milliseconds>(end - start)
      .count();
}

std::string trim(const std::string& value) {
  const auto begin = value.find_first_not_of(" \t\r\n");
  if (begin == std::string::npos) {
    return "";
  }
  const auto end = value.find_last_not_of(" \t\r\n");
  return value.substr(begin, end - begin + 1);
}

std::string lower_ascii(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(),
                 [](unsigned char c) {
                   return static_cast<char>(std::tolower(c));
                 });
  return value;
}

bool contains_lower(const std::string& haystack, const std::string& needle) {
  return lower_ascii(haystack).find(lower_ascii(needle)) != std::string::npos;
}

std::vector<std::string> available_ort_providers() {
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
  try {
    return Ort::GetAvailableProviders();
  } catch (...) {
    return {};
  }
#else
  return {};
#endif
}

bool provider_available(const std::vector<std::string>& providers,
                        const std::string& token) {
  return std::any_of(providers.begin(), providers.end(),
                     [&](const std::string& provider) {
                       return contains_lower(provider, token);
                     });
}

int android_device_api_level() {
#if defined(__ANDROID__)
  return android_get_device_api_level();
#else
  return 0;
#endif
}

std::string android_system_property(const char* key) {
#if defined(__ANDROID__)
  char value[PROP_VALUE_MAX] = {};
  const int length = __system_property_get(key, value);
  return length > 0 ? std::string(value, static_cast<size_t>(length)) : "";
#else
  (void)key;
  return "";
#endif
}

std::string android_soc_summary() {
#if defined(__ANDROID__)
  std::vector<std::string> parts;
  for (const char* key :
       {"ro.soc.manufacturer", "ro.soc.model", "ro.hardware",
        "ro.board.platform", "ro.product.board"}) {
    const std::string value = android_system_property(key);
    if (!value.empty()) {
      parts.push_back(std::string(key) + "=" + value);
    }
  }
  if (parts.empty()) {
    return "";
  }
  std::ostringstream out;
  for (size_t i = 0; i < parts.size(); ++i) {
    if (i > 0) {
      out << "; ";
    }
    out << parts[i];
  }
  return out.str();
#else
  return "";
#endif
}

bool android_reports_qualcomm_soc() {
#if defined(__ANDROID__)
  const std::string summary = lower_ascii(android_soc_summary());
  if (summary.find("qualcomm") != std::string::npos ||
      summary.find("snapdragon") != std::string::npos ||
      summary.find("qcom") != std::string::npos) {
    return true;
  }
  for (const std::string& marker : {"msm", "sdm", "sm8", "sm7", "sm6"}) {
    if (summary.find(marker) != std::string::npos) {
      return true;
    }
  }
#endif
  return false;
}

std::string backend_display_name(agus_ocr_backend_t backend) {
  switch (backend) {
    case AGUS_OCR_BACKEND_CPU:
      return "CPU";
    case AGUS_OCR_BACKEND_XNNPACK:
      return "XNNPACK";
    case AGUS_OCR_BACKEND_NNAPI:
      return "NNAPI";
    case AGUS_OCR_BACKEND_QNN:
      return "QNN";
    case AGUS_OCR_BACKEND_DIRECTML:
      return "DirectML";
    case AGUS_OCR_BACKEND_CUDA:
      return "CUDA";
    default:
      return backend_label(backend);
  }
}

AcceleratorStatus detect_gemma_cuda_status() {
  AcceleratorStatus status;
  status.backend = AGUS_OCR_BACKEND_CUDA;
  status.device_name = "ONNX Runtime CUDA device 0";
  status.unavailable_reason =
      "ONNX Runtime CUDA is available only on Windows native builds.";
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE) && defined(_WIN32)
  gemma_backend_is_unhealthy(AGUS_OCR_BACKEND_CUDA, &status.last_failure);
  if (!status.last_failure.empty()) {
    status.unavailable_reason =
        "CUDA was disabled after a Gemma runtime failure: " +
        status.last_failure;
    return status;
  }
  try {
    Ort::SessionOptions options;
    OrtCUDAProviderOptions cuda_options{};
    cuda_options.device_id = 0;
    const auto append_cuda =
        Ort::GetApi().SessionOptionsAppendExecutionProvider_CUDA;
    if (append_cuda == nullptr) {
      status.unavailable_reason =
          "This ONNX Runtime build does not expose the CUDA provider API.";
      return status;
    }
    Ort::ThrowOnError(append_cuda(options, &cuda_options));
    status.supported = true;
    status.unavailable_reason.clear();
  } catch (const std::exception& error) {
    status.unavailable_reason =
        "CUDA EP probe failed. Ensure onnxruntime_providers_cuda.dll, "
        "CUDA 12 runtime DLLs, and cuDNN 9 DLLs are beside the app: " +
        std::string(error.what());
  }
#endif
  return status;
}

void populate_generative_capabilities(RuntimeCapabilities* capabilities) {
  capabilities->generative_accelerators.clear();
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
  for (const auto& info : agus_ocr::LlamaVlmBackendInfos()) {
    GenerativeAcceleratorStatus status;
    status.backend = info.backend;
    status.supported = info.supported;
    status.enabled_by_default = info.enabled_by_default;
    status.device_name = info.device_name;
    status.unavailable_reason = info.unavailable_reason;
    status.last_failure = info.last_failure;
    capabilities->generative_accelerators.push_back(std::move(status));
  }
#else
  GenerativeAcceleratorStatus cpu;
  cpu.backend = AGUS_OCR_GEN_BACKEND_CPU;
  cpu.unavailable_reason =
      "PaddleOCR-VL native llama.cpp runtime is not built for this target.";
  capabilities->generative_accelerators.push_back(std::move(cpu));
#endif
  if (capabilities->generative_accelerators.empty()) {
    GenerativeAcceleratorStatus cpu;
    cpu.backend = AGUS_OCR_GEN_BACKEND_CPU;
    cpu.unavailable_reason =
        "No PaddleOCR-VL generative backend was reported by the native runtime.";
    capabilities->generative_accelerators.push_back(std::move(cpu));
  }
}

RuntimeCapabilities detect_runtime_capabilities() {
  RuntimeCapabilities capabilities;
  capabilities.runtime_summary =
      make_runtime_summary(capabilities.default_backend, 0);
  populate_generative_capabilities(&capabilities);
  AcceleratorStatus directml_status;
  directml_status.backend = AGUS_OCR_BACKEND_DIRECTML;
  directml_status.unavailable_reason =
      "DirectML is not available on this platform.";
#if defined(_WIN32)
  AcceleratorStatus cuda_status = detect_gemma_cuda_status();
#endif

#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE) && defined(__ANDROID__)
  (void)directml_status;
  capabilities.directml_unavailable_reason =
      "DirectML is available only on Windows native builds.";
  const std::vector<std::string> providers = available_ort_providers();
  const int api_level = android_device_api_level();
  const std::string soc_summary = android_soc_summary();
  const bool is_qualcomm = android_reports_qualcomm_soc();

  AcceleratorStatus cpu;
  cpu.backend = AGUS_OCR_BACKEND_CPU;
  cpu.supported = true;
  cpu.device_name = "ONNX Runtime CPU";

  AcceleratorStatus xnnpack;
  xnnpack.backend = AGUS_OCR_BACKEND_XNNPACK;
  xnnpack.device_name = "ONNX Runtime XNNPACK CPU";
  xnnpack.supported = provider_available(providers, "xnnpack");
  backend_is_unhealthy(AGUS_OCR_BACKEND_XNNPACK, &xnnpack.last_failure);
  if (!xnnpack.supported) {
    xnnpack.unavailable_reason =
        "XNNPACK EP was not reported by the packaged ONNX Runtime build.";
  } else if (!xnnpack.last_failure.empty()) {
    xnnpack.supported = false;
    xnnpack.unavailable_reason =
        "XNNPACK was disabled after a runtime failure: " +
        xnnpack.last_failure;
  }

  AcceleratorStatus nnapi;
  nnapi.backend = AGUS_OCR_BACKEND_NNAPI;
  nnapi.device_name =
      "Android NNAPI API " + std::to_string(api_level);
  nnapi.supported =
      api_level >= 27 && provider_available(providers, "nnapi");
  backend_is_unhealthy(AGUS_OCR_BACKEND_NNAPI, &nnapi.last_failure);
  if (api_level < 27) {
    nnapi.unavailable_reason =
        "NNAPI EP requires Android 8.1 / API 27 or newer.";
  } else if (!provider_available(providers, "nnapi")) {
    nnapi.unavailable_reason =
        "NNAPI EP was not reported by the packaged ONNX Runtime build.";
  } else if (!nnapi.last_failure.empty()) {
    nnapi.supported = false;
    nnapi.unavailable_reason =
        "NNAPI was disabled after a runtime failure: " + nnapi.last_failure;
  }

  AcceleratorStatus qnn;
  qnn.backend = AGUS_OCR_BACKEND_QNN;
  qnn.device_name =
      soc_summary.empty() ? "Qualcomm QNN GPU backend" : soc_summary;
  qnn.supported =
      provider_available(providers, "qnn") && is_qualcomm;
  backend_is_unhealthy(AGUS_OCR_BACKEND_QNN, &qnn.last_failure);
  if (!provider_available(providers, "qnn")) {
    qnn.unavailable_reason =
        "QNN EP was not reported by the packaged ONNX Runtime build.";
  } else if (!is_qualcomm) {
    qnn.unavailable_reason =
        "QNN EP is packaged, but this device does not report a Qualcomm/Snapdragon SoC.";
  } else if (!qnn.last_failure.empty()) {
    qnn.supported = false;
    qnn.unavailable_reason =
        "QNN was disabled after a runtime failure: " + qnn.last_failure;
  }

  capabilities.default_backend = AGUS_OCR_BACKEND_CPU;
  for (const auto backend :
       {AGUS_OCR_BACKEND_QNN, AGUS_OCR_BACKEND_NNAPI,
        AGUS_OCR_BACKEND_XNNPACK}) {
    const AcceleratorStatus* status = backend == AGUS_OCR_BACKEND_QNN
                                          ? &qnn
                                          : backend == AGUS_OCR_BACKEND_NNAPI
                                                ? &nnapi
                                                : &xnnpack;
    if (status->supported) {
      capabilities.default_backend = backend;
      break;
    }
  }
  cpu.enabled_by_default = capabilities.default_backend == AGUS_OCR_BACKEND_CPU;
  xnnpack.enabled_by_default =
      capabilities.default_backend == AGUS_OCR_BACKEND_XNNPACK;
  nnapi.enabled_by_default =
      capabilities.default_backend == AGUS_OCR_BACKEND_NNAPI;
  qnn.enabled_by_default =
      capabilities.default_backend == AGUS_OCR_BACKEND_QNN;

  capabilities.accelerators = {std::move(cpu), std::move(xnnpack),
                               std::move(nnapi), std::move(qnn)};
  capabilities.runtime_summary =
      make_runtime_summary(capabilities.default_backend, 0);
  return capabilities;
#endif

#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE) && defined(_WIN32) && \
    defined(AGUS_OCR_ENABLE_DIRECTML)
  std::string unhealthy_reason;
  const bool unhealthy = directml_is_unhealthy(&unhealthy_reason);
  IDXGIFactory1* factory = nullptr;
  const HRESULT factory_result =
      CreateDXGIFactory1(__uuidof(IDXGIFactory1),
                         reinterpret_cast<void**>(&factory));
  if (FAILED(factory_result) || factory == nullptr) {
    capabilities.directml_unavailable_reason =
        "DXGI factory creation failed; DirectML hardware detection is unavailable.";
    directml_status.unavailable_reason =
        capabilities.directml_unavailable_reason;
    directml_status.last_failure = unhealthy_reason;
    capabilities.accelerators.push_back(directml_status);
    capabilities.accelerators.push_back(cuda_status);
    return capabilities;
  }

  IDXGIAdapter1* adapter = nullptr;
  for (UINT index = 0; factory->EnumAdapters1(index, &adapter) !=
                       DXGI_ERROR_NOT_FOUND;
       ++index) {
    DXGI_ADAPTER_DESC1 description{};
    if (SUCCEEDED(adapter->GetDesc1(&description)) &&
        (description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) == 0) {
      const HRESULT device_result =
          D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0,
                            __uuidof(ID3D12Device), nullptr);
      if (SUCCEEDED(device_result)) {
        int required = WideCharToMultiByte(CP_UTF8, 0, description.Description,
                                           -1, nullptr, 0, nullptr, nullptr);
        if (required > 1) {
          std::string name(static_cast<size_t>(required - 1), '\0');
          WideCharToMultiByte(CP_UTF8, 0, description.Description, -1,
                              name.data(), required, nullptr, nullptr);
          capabilities.directml_device_name = name;
          directml_status.device_name = name;
        }
        directml_status.supported = !unhealthy;
        directml_status.enabled_by_default = !unhealthy;
        directml_status.last_failure = unhealthy_reason;
        if (unhealthy) {
          capabilities.directml_supported = false;
          capabilities.directml_enabled_by_default = false;
          capabilities.default_backend = AGUS_OCR_BACKEND_CPU;
          capabilities.directml_unavailable_reason =
              "DirectML was disabled after a runtime failure: " +
              unhealthy_reason;
          directml_status.unavailable_reason =
              capabilities.directml_unavailable_reason;
        } else {
          capabilities.directml_supported = true;
          capabilities.directml_enabled_by_default = true;
          capabilities.default_backend = AGUS_OCR_BACKEND_DIRECTML;
          capabilities.directml_unavailable_reason.clear();
          directml_status.unavailable_reason.clear();
        }
        adapter->Release();
        factory->Release();
        capabilities.runtime_summary =
            make_runtime_summary(capabilities.default_backend, 0);
        capabilities.accelerators.push_back(directml_status);
        capabilities.accelerators.push_back(cuda_status);
        return capabilities;
      }
    }
    adapter->Release();
    adapter = nullptr;
  }

  factory->Release();
  capabilities.directml_unavailable_reason =
      "No hardware DirectX 12 adapter was found for DirectML.";
  directml_status.unavailable_reason =
      capabilities.directml_unavailable_reason;
  directml_status.last_failure = unhealthy_reason;
#elif defined(_WIN32)
  capabilities.directml_unavailable_reason =
      "This build was compiled without DirectML support.";
  directml_status.unavailable_reason =
      capabilities.directml_unavailable_reason;
#endif

  capabilities.accelerators.push_back(directml_status);
#if defined(_WIN32)
  capabilities.accelerators.push_back(cuda_status);
#endif
  return capabilities;
}

#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)

struct Tensor {
  std::vector<float> data;
  std::vector<int64_t> shape;
};

struct OcrPoint {
  float x = 0;
  float y = 0;
};

struct OcrLine {
  std::string text;
  float confidence = 0;
  std::vector<OcrPoint> polygon;
  int textline_angle = -1;
};

Ort::Env& ort_env() {
  static Ort::Env env(ORT_LOGGING_LEVEL_WARNING, "agus_docs_ocr");
  return env;
}

bool backend_supported_for_ppocr(agus_ocr_backend_t backend,
                                 std::string* unavailable_reason = nullptr) {
  if (backend == AGUS_OCR_BACKEND_CPU) {
    return true;
  }
  if (backend == AGUS_OCR_BACKEND_CUDA) {
    if (unavailable_reason != nullptr) {
      *unavailable_reason =
          "CUDA is only available to Gemma Markdown in this build.";
    }
    return false;
  }
  const RuntimeCapabilities capabilities = detect_runtime_capabilities();
  for (const AcceleratorStatus& accelerator : capabilities.accelerators) {
    if (accelerator.backend == backend) {
      if (unavailable_reason != nullptr) {
        *unavailable_reason = accelerator.unavailable_reason;
      }
      return accelerator.supported;
    }
  }
  if (unavailable_reason != nullptr) {
    *unavailable_reason =
        backend_display_name(backend) + " is not available on this platform.";
  }
  return false;
}

bool backend_supported_for_gemma(agus_ocr_backend_t backend,
                                 std::string* unavailable_reason = nullptr) {
  if (backend == AGUS_OCR_BACKEND_CPU) {
    return true;
  }
  if (backend != AGUS_OCR_BACKEND_DIRECTML &&
      backend != AGUS_OCR_BACKEND_CUDA) {
    if (unavailable_reason != nullptr) {
      *unavailable_reason = backend_display_name(backend) +
                            " is not a Gemma Markdown ONNX backend.";
    }
    return false;
  }
  const RuntimeCapabilities capabilities = detect_runtime_capabilities();
  for (const AcceleratorStatus& accelerator : capabilities.accelerators) {
    if (accelerator.backend == backend) {
      if (unavailable_reason != nullptr) {
        *unavailable_reason = accelerator.unavailable_reason;
      }
      return accelerator.supported;
    }
  }
  if (unavailable_reason != nullptr) {
    *unavailable_reason =
        backend_display_name(backend) + " is not available on this platform.";
  }
  return false;
}

bool is_onnx_accelerator_backend(agus_ocr_backend_t backend) {
  return backend == AGUS_OCR_BACKEND_DIRECTML ||
         backend == AGUS_OCR_BACKEND_XNNPACK ||
         backend == AGUS_OCR_BACKEND_NNAPI ||
         backend == AGUS_OCR_BACKEND_QNN;
}

std::vector<agus_ocr_backend_t> ppocr_backend_attempts(
    const RuntimeOptions& runtime) {
  if (runtime.force_cpu_only || runtime.backend == AGUS_OCR_BACKEND_CPU) {
    return {AGUS_OCR_BACKEND_CPU};
  }
  if (runtime.backend != AGUS_OCR_BACKEND_AUTO) {
    if (is_onnx_accelerator_backend(runtime.backend)) {
      return {runtime.backend, AGUS_OCR_BACKEND_CPU};
    }
    return {AGUS_OCR_BACKEND_CPU};
  }

#if defined(__ANDROID__)
  std::vector<agus_ocr_backend_t> out;
  for (const auto backend :
       {AGUS_OCR_BACKEND_QNN, AGUS_OCR_BACKEND_NNAPI,
        AGUS_OCR_BACKEND_XNNPACK}) {
    std::string reason;
    if (backend_supported_for_ppocr(backend, &reason)) {
      out.push_back(backend);
    }
  }
  out.push_back(AGUS_OCR_BACKEND_CPU);
  return out;
#else
  const RuntimeCapabilities capabilities = detect_runtime_capabilities();
  if (capabilities.default_backend == AGUS_OCR_BACKEND_DIRECTML) {
    return {AGUS_OCR_BACKEND_DIRECTML, AGUS_OCR_BACKEND_CPU};
  }
  return {AGUS_OCR_BACKEND_CPU};
#endif
}

std::vector<agus_ocr_backend_t> gemma_backend_attempts(
    const RuntimeOptions& runtime) {
  if (runtime.force_cpu_only || runtime.backend == AGUS_OCR_BACKEND_CPU) {
    return {AGUS_OCR_BACKEND_CPU};
  }

  std::vector<agus_ocr_backend_t> attempts;
  auto add_if_candidate = [&attempts](agus_ocr_backend_t backend) {
    if (std::find(attempts.begin(), attempts.end(), backend) ==
        attempts.end()) {
      attempts.push_back(backend);
    }
  };

  if (runtime.backend != AGUS_OCR_BACKEND_AUTO) {
    if (runtime.backend == AGUS_OCR_BACKEND_DIRECTML ||
        runtime.backend == AGUS_OCR_BACKEND_CUDA) {
      add_if_candidate(runtime.backend);
    }
    add_if_candidate(AGUS_OCR_BACKEND_CPU);
  } else {
    for (const auto backend :
         {AGUS_OCR_BACKEND_CUDA, AGUS_OCR_BACKEND_DIRECTML}) {
      std::string reason;
      if (backend_supported_for_gemma(backend, &reason)) {
        add_if_candidate(backend);
      }
    }
    add_if_candidate(AGUS_OCR_BACKEND_CPU);
  }

  attempts.erase(
      std::remove_if(attempts.begin(), attempts.end(), [](auto backend) {
        return backend != AGUS_OCR_BACKEND_CPU &&
               gemma_backend_is_unhealthy(backend);
      }),
      attempts.end());
  if (attempts.empty()) {
    attempts.push_back(AGUS_OCR_BACKEND_CPU);
  }
  return attempts;
}

agus_ocr_backend_t resolve_backend(const RuntimeOptions& runtime) {
  if (runtime.force_cpu_only) {
    return AGUS_OCR_BACKEND_CPU;
  }
  if (runtime.backend == AGUS_OCR_BACKEND_DIRECTML ||
      runtime.backend == AGUS_OCR_BACKEND_XNNPACK ||
      runtime.backend == AGUS_OCR_BACKEND_NNAPI ||
      runtime.backend == AGUS_OCR_BACKEND_QNN) {
    if (runtime.backend == AGUS_OCR_BACKEND_DIRECTML) {
      std::string unhealthy_reason;
      if (directml_is_unhealthy(&unhealthy_reason)) {
        throw std::runtime_error(
            "DirectML was disabled after a runtime failure: " +
            unhealthy_reason);
      }
      const RuntimeCapabilities capabilities = detect_runtime_capabilities();
      if (!capabilities.directml_supported) {
        throw std::runtime_error("DirectML requested but unavailable: " +
                                 capabilities.directml_unavailable_reason);
      }
    } else {
      std::string reason;
      if (!backend_supported_for_ppocr(runtime.backend, &reason)) {
        throw std::runtime_error(backend_display_name(runtime.backend) +
                                 " requested but unavailable: " + reason);
      }
    }
    std::string unhealthy_reason;
    if (backend_is_unhealthy(runtime.backend, &unhealthy_reason)) {
      throw std::runtime_error(backend_display_name(runtime.backend) +
                               " was disabled after a runtime failure: " +
                               unhealthy_reason);
    }
    return runtime.backend;
  }
  if (runtime.backend == AGUS_OCR_BACKEND_AUTO) {
    const std::vector<agus_ocr_backend_t> attempts =
        ppocr_backend_attempts(runtime);
    return attempts.empty() ? AGUS_OCR_BACKEND_CPU : attempts.front();
  }
  return AGUS_OCR_BACKEND_CPU;
}

#if defined(_WIN32)
std::wstring utf8_to_wide(const std::string& value) {
  if (value.empty()) {
    return std::wstring();
  }
  const int required = MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1,
                                           nullptr, 0);
  if (required <= 0) {
    throw std::runtime_error("failed to convert UTF-8 path to UTF-16");
  }
  std::wstring out(static_cast<size_t>(required - 1), L'\0');
  MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, out.data(), required);
  return out;
}
#endif

class OrtRunner {
 public:
  OrtRunner(const std::string& model_path, const RuntimeOptions& runtime,
            const std::string& session_name, agus_ocr_backend_t active_backend) {
    Ort::SessionOptions options;
    options.SetGraphOptimizationLevel(GraphOptimizationLevel::ORT_ENABLE_ALL);
    const int cpu_threads = effective_cpu_threads(runtime);
    if (active_backend == AGUS_OCR_BACKEND_XNNPACK) {
      options.SetIntraOpNumThreads(1);
      options.SetInterOpNumThreads(1);
      options.SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
      options.AddConfigEntry("session.intra_op.allow_spinning", "0");
    } else if (cpu_threads > 0) {
      options.SetIntraOpNumThreads(cpu_threads);
      options.SetInterOpNumThreads(1);
      options.SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
    }
    {
      std::ostringstream message;
      message << "core OrtRunner session=" << session_name
              << " backend=" << backend_label(active_backend)
              << " effectiveCpuThreads=" << cpu_threads;
      core_log_info(message.str());
    }
    if (runtime.enable_ort_profiling) {
#if defined(_WIN32)
      const std::wstring wide_session_name = utf8_to_wide(session_name);
      options.EnableProfiling(wide_session_name.c_str());
#else
      options.EnableProfiling(session_name.c_str());
#endif
    }
    if (active_backend == AGUS_OCR_BACKEND_DIRECTML) {
#if defined(_WIN32) && defined(AGUS_OCR_ENABLE_DIRECTML)
      options.DisableMemPattern();
      options.SetExecutionMode(ExecutionMode::ORT_SEQUENTIAL);
      Ort::ThrowOnError(OrtSessionOptionsAppendExecutionProvider_DML(options, 0));
#else
      throw std::runtime_error("DirectML backend was selected in a non-DirectML build");
#endif
    } else if (active_backend == AGUS_OCR_BACKEND_XNNPACK) {
      const int xnnpack_threads = std::max(1, cpu_threads);
      options.AppendExecutionProvider(
          "XNNPACK",
          std::unordered_map<std::string, std::string>{
              {"intra_op_num_threads", std::to_string(xnnpack_threads)}});
    } else if (active_backend == AGUS_OCR_BACKEND_NNAPI) {
#if defined(__ANDROID__) && defined(AGUS_OCR_HAS_NNAPI_PROVIDER_FACTORY)
      uint32_t nnapi_flags = 0;
      Ort::ThrowOnError(
          OrtSessionOptionsAppendExecutionProvider_Nnapi(options, nnapi_flags));
#else
      throw std::runtime_error("NNAPI backend was selected in a non-NNAPI build");
#endif
    } else if (active_backend == AGUS_OCR_BACKEND_QNN) {
      options.AppendExecutionProvider(
          "QNN", std::unordered_map<std::string, std::string>{
                     {"backend_type", "gpu"}});
    }
#if defined(_WIN32)
    const std::wstring wide_model_path = utf8_to_wide(model_path);
    session_ = std::make_unique<Ort::Session>(
        ort_env(), wide_model_path.c_str(), options);
#else
    session_ =
        std::make_unique<Ort::Session>(ort_env(), model_path.c_str(), options);
#endif

    Ort::AllocatorWithDefaultOptions allocator;
    const size_t input_count = session_->GetInputCount();
    input_names_.reserve(input_count);
    input_name_ptrs_.reserve(input_count);
    for (size_t i = 0; i < input_count; ++i) {
      auto name = session_->GetInputNameAllocated(i, allocator);
      input_names_.push_back(name.get());
    }
    for (const auto& name : input_names_) {
      input_name_ptrs_.push_back(name.c_str());
    }

    const size_t output_count = session_->GetOutputCount();
    output_names_.reserve(output_count);
    output_name_ptrs_.reserve(output_count);
    for (size_t i = 0; i < output_count; ++i) {
      auto name = session_->GetOutputNameAllocated(i, allocator);
      output_names_.push_back(name.get());
    }
    for (const auto& name : output_names_) {
      output_name_ptrs_.push_back(name.c_str());
    }
  }

  Tensor run(const std::vector<float>& input,
             const std::vector<int64_t>& shape) const {
    Ort::MemoryInfo memory_info =
        Ort::MemoryInfo::CreateCpu(OrtArenaAllocator, OrtMemTypeDefault);
    const size_t count =
        static_cast<size_t>(std::accumulate(shape.begin(), shape.end(), int64_t{1},
                                            std::multiplies<int64_t>()));
    if (count != input.size()) {
      throw std::runtime_error("input tensor shape does not match data length");
    }

    auto tensor = Ort::Value::CreateTensor<float>(
        memory_info, const_cast<float*>(input.data()), input.size(),
        shape.data(), shape.size());
    auto outputs = session_->Run(Ort::RunOptions{nullptr}, input_name_ptrs_.data(),
                                 &tensor, 1, output_name_ptrs_.data(),
                                 output_name_ptrs_.size());
    if (outputs.empty() || !outputs[0].IsTensor()) {
      throw std::runtime_error("ONNX model did not return a tensor");
    }
    auto type_info = outputs[0].GetTensorTypeAndShapeInfo();
    Tensor out;
    out.shape = type_info.GetShape();
    const size_t output_count = type_info.GetElementCount();
    const float* data = outputs[0].GetTensorData<float>();
    out.data.assign(data, data + output_count);
    return out;
  }

 private:
  std::unique_ptr<Ort::Session> session_;
  std::vector<std::string> input_names_;
  std::vector<std::string> output_names_;
  std::vector<const char*> input_name_ptrs_;
  std::vector<const char*> output_name_ptrs_;
};

std::string parse_yaml_scalar(std::string value) {
  value = trim(value);
  const auto comment = value.find(" #");
  if (comment != std::string::npos) {
    value = trim(value.substr(0, comment));
  }
  if (value.size() >= 2 && value.front() == '\'' && value.back() == '\'') {
    std::string inner = value.substr(1, value.size() - 2);
    std::string out;
    for (size_t i = 0; i < inner.size(); ++i) {
      if (inner[i] == '\'' && i + 1 < inner.size() && inner[i + 1] == '\'') {
        out.push_back('\'');
        ++i;
      } else {
        out.push_back(inner[i]);
      }
    }
    return out;
  }
  if (value.size() >= 2 && value.front() == '"' && value.back() == '"') {
    return value.substr(1, value.size() - 2);
  }
  return value;
}

std::string base64_encode(const std::vector<uchar>& bytes) {
  static constexpr char kAlphabet[] =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  std::string encoded;
  encoded.reserve(((bytes.size() + 2) / 3) * 4);
  size_t i = 0;
  while (i + 2 < bytes.size()) {
    const uint32_t value = (static_cast<uint32_t>(bytes[i]) << 16) |
                           (static_cast<uint32_t>(bytes[i + 1]) << 8) |
                           static_cast<uint32_t>(bytes[i + 2]);
    encoded.push_back(kAlphabet[(value >> 18) & 0x3f]);
    encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
    encoded.push_back(kAlphabet[(value >> 6) & 0x3f]);
    encoded.push_back(kAlphabet[value & 0x3f]);
    i += 3;
  }
  if (i < bytes.size()) {
    uint32_t value = static_cast<uint32_t>(bytes[i]) << 16;
    encoded.push_back(kAlphabet[(value >> 18) & 0x3f]);
    if (i + 1 < bytes.size()) {
      value |= static_cast<uint32_t>(bytes[i + 1]) << 8;
      encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
      encoded.push_back(kAlphabet[(value >> 6) & 0x3f]);
      encoded.push_back('=');
    } else {
      encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
      encoded.push_back('=');
      encoded.push_back('=');
    }
  }
  return encoded;
}

std::string encode_overlay_image_base64(const cv::Mat& page,
                                        std::string* mime_type) {
  std::vector<uchar> encoded;
  std::vector<int> params = {cv::IMWRITE_JPEG_QUALITY, 90};
  if (cv::imencode(".jpg", page, encoded, params)) {
    *mime_type = "image/jpeg";
    return base64_encode(encoded);
  }
  if (cv::imencode(".png", page, encoded)) {
    *mime_type = "image/png";
    return base64_encode(encoded);
  }
  throw std::runtime_error("failed to encode overlay image");
}

enum class AnnotationGeometry {
  kNone = 0,
  kExact = 1,
  kEstimated = 2,
};

struct MappingEstimate {
  bool available = false;
  AnnotationGeometry geometry = AnnotationGeometry::kNone;
  double confidence = 0.0;
  std::string message;
  cv::Mat processed_to_source;
};

cv::Mat affine_to_homography(const cv::Mat& affine) {
  cv::Mat homography = cv::Mat::eye(3, 3, CV_64F);
  for (int y = 0; y < 2; ++y) {
    for (int x = 0; x < 3; ++x) {
      homography.at<double>(y, x) = affine.at<double>(y, x);
    }
  }
  return homography;
}

cv::Point2f project_point(const cv::Mat& homography,
                          const cv::Point2f& point) {
  const double x = point.x;
  const double y = point.y;
  const double z = homography.at<double>(2, 0) * x +
                   homography.at<double>(2, 1) * y +
                   homography.at<double>(2, 2);
  if (std::abs(z) < 1e-6) {
    return {std::numeric_limits<float>::quiet_NaN(),
            std::numeric_limits<float>::quiet_NaN()};
  }
  return {static_cast<float>((homography.at<double>(0, 0) * x +
                              homography.at<double>(0, 1) * y +
                              homography.at<double>(0, 2)) /
                             z),
          static_cast<float>((homography.at<double>(1, 0) * x +
                              homography.at<double>(1, 1) * y +
                              homography.at<double>(1, 2)) /
                             z)};
}

float clamp_float(float value, float min_value, float max_value) {
  return std::max(min_value, std::min(value, max_value));
}

OcrLine transform_line(const OcrLine& line, const cv::Mat& homography,
                       const cv::Size& target_size) {
  OcrLine transformed = line;
  transformed.polygon.clear();
  transformed.polygon.reserve(line.polygon.size());
  const float max_x = static_cast<float>(std::max(0, target_size.width - 1));
  const float max_y = static_cast<float>(std::max(0, target_size.height - 1));
  for (const auto& point : line.polygon) {
    const cv::Point2f projected =
        project_point(homography, {point.x, point.y});
    if (!std::isfinite(projected.x) || !std::isfinite(projected.y)) {
      transformed.polygon.push_back({0.0f, 0.0f});
      continue;
    }
    transformed.polygon.push_back(
        {clamp_float(projected.x, 0.0f, max_x),
         clamp_float(projected.y, 0.0f, max_y)});
  }
  return transformed;
}

std::vector<OcrLine> transform_lines(const std::vector<OcrLine>& lines,
                                     const cv::Mat& homography,
                                     const cv::Size& target_size) {
  std::vector<OcrLine> transformed;
  transformed.reserve(lines.size());
  for (const auto& line : lines) {
    transformed.push_back(transform_line(line, homography, target_size));
  }
  return transformed;
}

std::pair<OcrPoint, OcrPoint> line_bounds(const OcrLine& line) {
  float left = std::numeric_limits<float>::max();
  float top = std::numeric_limits<float>::max();
  float right = std::numeric_limits<float>::lowest();
  float bottom = std::numeric_limits<float>::lowest();
  for (const auto& point : line.polygon) {
    left = std::min(left, point.x);
    top = std::min(top, point.y);
    right = std::max(right, point.x);
    bottom = std::max(bottom, point.y);
  }
  if (line.polygon.empty()) {
    return {{0.0f, 0.0f}, {0.0f, 0.0f}};
  }
  return {{left, top}, {right, bottom}};
}

void append_lines_json(std::ostringstream* out,
                       const std::vector<OcrLine>& lines) {
  *out << '[';
  for (size_t i = 0; i < lines.size(); ++i) {
    const auto& line = lines[i];
    if (i > 0) {
      *out << ',';
    }
    const auto bounds = line_bounds(line);
    *out << "{\"text\":\"" << json_escape(line.text)
         << "\",\"confidence\":" << line.confidence << ",\"polygon\":[";
    for (size_t p = 0; p < line.polygon.size(); ++p) {
      if (p > 0) {
        *out << ',';
      }
      *out << "{\"x\":" << line.polygon[p].x << ",\"y\":"
           << line.polygon[p].y << "}";
    }
    *out << "],\"boundingBox\":{\"left\":" << bounds.first.x
         << ",\"top\":" << bounds.first.y
         << ",\"right\":" << bounds.second.x
         << ",\"bottom\":" << bounds.second.y
         << "},\"textLineAngle\":" << line.textline_angle << "}";
  }
  *out << ']';
}

std::string markdown_from_lines(const std::vector<OcrLine>& lines) {
  std::ostringstream out;
  for (size_t i = 0; i < lines.size(); ++i) {
    if (i > 0) {
      out << '\n';
    }
    out << lines[i].text;
  }
  return out.str();
}

void append_string_array_json(std::ostringstream* out,
                              const std::vector<std::string>& values) {
  *out << '[';
  for (size_t i = 0; i < values.size(); ++i) {
    if (i > 0) {
      *out << ',';
    }
    *out << '"' << json_escape(values[i]) << '"';
  }
  *out << ']';
}

void append_blocks_json(std::ostringstream* out,
                        const std::vector<OcrLine>& lines,
                        const std::string& source_layer_id) {
  *out << '[';
  for (size_t i = 0; i < lines.size(); ++i) {
    const auto& line = lines[i];
    if (i > 0) {
      *out << ',';
    }
    const auto bounds = line_bounds(line);
    *out << "{\"id\":\"line-" << i
         << "\",\"label\":\"text\",\"text\":\"" << json_escape(line.text)
         << "\",\"markdown\":\"" << json_escape(line.text)
         << "\",\"confidence\":" << line.confidence
         << ",\"readingOrder\":" << i
         << ",\"polygon\":[";
    for (size_t p = 0; p < line.polygon.size(); ++p) {
      if (p > 0) {
        *out << ',';
      }
      *out << "{\"x\":" << line.polygon[p].x << ",\"y\":"
           << line.polygon[p].y << "}";
    }
    *out << "],\"boundingBox\":{\"left\":" << bounds.first.x
         << ",\"top\":" << bounds.first.y
         << ",\"right\":" << bounds.second.x
         << ",\"bottom\":" << bounds.second.y
         << "},\"sourceLayerId\":\"" << json_escape(source_layer_id)
         << "\"}";
  }
  *out << ']';
}

void append_annotation_layer_json(std::ostringstream* out,
                                  const std::string& id,
                                  const std::string& label,
                                  const cv::Mat& image,
                                  const std::vector<OcrLine>& lines,
                                  AnnotationGeometry geometry,
                                  bool is_available,
                                  double confidence,
                                  const std::string& message) {
  std::string mime_type;
  const std::string image_base64 = encode_overlay_image_base64(image, &mime_type);
  *out << "{\"id\":\"" << json_escape(id)
       << "\",\"label\":\"" << json_escape(label)
       << "\",\"width\":" << image.cols
       << ",\"height\":" << image.rows
       << ",\"imageMimeType\":\"" << mime_type
       << "\",\"imageBytesBase64\":\"" << image_base64
       << "\",\"lines\":";
  append_lines_json(out, lines);
  *out << ",\"geometry\":" << static_cast<int>(geometry)
       << ",\"isAvailable\":" << (is_available ? "true" : "false")
       << ",\"confidence\":" << confidence
       << ",\"message\":\"" << json_escape(message) << "\"}";
}

std::vector<std::string> load_character_dict(const std::string& yml_path) {
  std::ifstream file(yml_path, std::ios::binary);
  if (!file) {
    throw std::runtime_error("failed to open recognition YAML: " + yml_path);
  }

  std::vector<std::string> characters;
  characters.push_back("blank");

  bool in_dict = false;
  std::string line;
  while (std::getline(file, line)) {
    const std::string stripped = trim(line);
    if (!in_dict) {
      if (stripped == "character_dict:") {
        in_dict = true;
      }
      continue;
    }
    if (stripped.rfind("- ", 0) == 0) {
      characters.push_back(parse_yaml_scalar(stripped.substr(2)));
      continue;
    }
    if (!stripped.empty() && line[0] != ' ' && line[0] != '\t') {
      break;
    }
  }

  if (characters.size() <= 1) {
    throw std::runtime_error("recognition character_dict is empty");
  }
  return characters;
}

cv::Mat decode_image(const agus_ocr_image_t& image) {
  cv::Mat encoded(1, static_cast<int>(image.length), CV_8UC1,
                  const_cast<uint8_t*>(image.bytes));
  cv::Mat decoded = cv::imdecode(encoded, cv::IMREAD_COLOR);
  if (decoded.empty()) {
    throw std::invalid_argument("image bytes could not be decoded");
  }
  return decoded;
}

cv::Mat normalize_input_image(const cv::Mat& image, const RunOptions& options,
                              std::vector<std::string>* warnings) {
  if (image.empty()) {
    return image;
  }
  const int64_t pixels =
      static_cast<int64_t>(image.cols) * static_cast<int64_t>(image.rows);
  const int64_t max_pixels =
      options.max_pixels > 0 ? options.max_pixels : int32_t{2500000};
  if (pixels <= max_pixels) {
    return image;
  }

  const double scale =
      std::sqrt(static_cast<double>(max_pixels) / static_cast<double>(pixels));
  const int target_w =
      std::max(1, static_cast<int>(std::round(image.cols * scale)));
  const int target_h =
      std::max(1, static_cast<int>(std::round(image.rows * scale)));
  cv::Mat resized;
  cv::resize(image, resized, cv::Size(target_w, target_h), 0, 0,
             cv::INTER_AREA);

  std::ostringstream message;
  message << "Input image normalized from " << image.cols << "x" << image.rows
          << " to " << resized.cols << "x" << resized.rows
          << " for bounded mobile OCR memory.";
  if (warnings != nullptr) {
    warnings->push_back(message.str());
  }
  core_log_info("core ppocr " + message.str());
  return resized;
}

std::vector<float> normalize_to_chw(const cv::Mat& bgr, bool rgb,
                                    const std::vector<float>& mean,
                                    const std::vector<float>& stddev,
                                    float scale) {
  if (bgr.empty() || bgr.channels() != 3) {
    throw std::invalid_argument("expected a non-empty three-channel image");
  }
  cv::Mat image;
  if (rgb) {
    cv::cvtColor(bgr, image, cv::COLOR_BGR2RGB);
  } else {
    image = bgr;
  }
  std::vector<float> out(static_cast<size_t>(3 * image.rows * image.cols));
  const int plane = image.rows * image.cols;
  for (int y = 0; y < image.rows; ++y) {
    const cv::Vec3b* row = image.ptr<cv::Vec3b>(y);
    for (int x = 0; x < image.cols; ++x) {
      const cv::Vec3b pixel = row[x];
      const int offset = y * image.cols + x;
      for (int c = 0; c < 3; ++c) {
        out[static_cast<size_t>(c * plane + offset)] =
            (static_cast<float>(pixel[c]) * scale - mean[c]) / stddev[c];
      }
    }
  }
  return out;
}

std::vector<float> scale_bgr_to_chw(const cv::Mat& bgr, float scale) {
  if (bgr.empty() || bgr.channels() != 3) {
    throw std::invalid_argument("expected a non-empty BGR image");
  }
  std::vector<float> out(static_cast<size_t>(3 * bgr.rows * bgr.cols));
  const int plane = bgr.rows * bgr.cols;
  for (int y = 0; y < bgr.rows; ++y) {
    const cv::Vec3b* row = bgr.ptr<cv::Vec3b>(y);
    for (int x = 0; x < bgr.cols; ++x) {
      const cv::Vec3b pixel = row[x];
      const int offset = y * bgr.cols + x;
      for (int c = 0; c < 3; ++c) {
        out[static_cast<size_t>(c * plane + offset)] =
            static_cast<float>(pixel[c]) * scale;
      }
    }
  }
  return out;
}

cv::Mat resize_short_and_crop(const cv::Mat& image, int short_size,
                              int crop_size) {
  const int h = image.rows;
  const int w = image.cols;
  const float scale =
      static_cast<float>(short_size) / static_cast<float>(std::min(h, w));
  const int new_h = std::max(1, static_cast<int>(std::round(h * scale)));
  const int new_w = std::max(1, static_cast<int>(std::round(w * scale)));
  cv::Mat resized;
  cv::resize(image, resized, cv::Size(new_w, new_h), 0, 0, cv::INTER_LINEAR);
  const int x = std::max(0, (resized.cols - crop_size) / 2);
  const int y = std::max(0, (resized.rows - crop_size) / 2);
  const int width = std::min(crop_size, resized.cols - x);
  const int height = std::min(crop_size, resized.rows - y);
  cv::Mat crop = resized(cv::Rect(x, y, width, height)).clone();
  if (crop.cols != crop_size || crop.rows != crop_size) {
    cv::resize(crop, crop, cv::Size(crop_size, crop_size));
  }
  return crop;
}

cv::Mat rotate_image_with_transform(const cv::Mat& image, int angle,
                                    cv::Mat* source_to_rotated) {
  if (angle < 0 || angle >= 360) {
    throw std::invalid_argument("angle should be in range [0, 360)");
  }
  if (angle == 0) {
    if (source_to_rotated != nullptr) {
      *source_to_rotated = (cv::Mat_<double>(2, 3) << 1.0, 0.0, 0.0, 0.0, 1.0,
                            0.0);
    }
    return image.clone();
  }
  const int h = image.rows;
  const int w = image.cols;
  const cv::Point2f center(w / 2.0f, h / 2.0f);
  cv::Mat rot = cv::getRotationMatrix2D(center, angle, 1.0);
  const double abs_cos = std::abs(rot.at<double>(0, 0));
  const double abs_sin = std::abs(rot.at<double>(0, 1));
  const int new_w = static_cast<int>(h * abs_sin + w * abs_cos);
  const int new_h = static_cast<int>(h * abs_cos + w * abs_sin);
  rot.at<double>(0, 2) += (new_w - w) / 2.0;
  rot.at<double>(1, 2) += (new_h - h) / 2.0;
  cv::Mat rotated;
  cv::warpAffine(image, rotated, rot, cv::Size(new_w, new_h),
                 cv::INTER_CUBIC);
  if (source_to_rotated != nullptr) {
    *source_to_rotated = rot;
  }
  return rotated;
}

cv::Mat rotate_image(const cv::Mat& image, int angle) {
  return rotate_image_with_transform(image, angle, nullptr);
}

double point_error(const cv::Point2f& a, const cv::Point2f& b) {
  const double dx = static_cast<double>(a.x) - static_cast<double>(b.x);
  const double dy = static_cast<double>(a.y) - static_cast<double>(b.y);
  return std::sqrt(dx * dx + dy * dy);
}

bool homography_is_usable(const cv::Mat& homography) {
  if (homography.empty() || homography.rows != 3 || homography.cols != 3) {
    return false;
  }
  for (int y = 0; y < homography.rows; ++y) {
    for (int x = 0; x < homography.cols; ++x) {
      if (!std::isfinite(homography.at<double>(y, x))) {
        return false;
      }
    }
  }
  return true;
}

MappingEstimate estimate_processed_to_oriented(const cv::Mat& processed,
                                               const cv::Mat& oriented) {
  MappingEstimate estimate;
  estimate.message = "Estimated original-image geometry is unavailable.";
  if (processed.empty() || oriented.empty()) {
    estimate.message = "Estimated geometry unavailable: image is empty.";
    return estimate;
  }

  cv::Mat processed_gray;
  cv::Mat oriented_gray;
  cv::cvtColor(processed, processed_gray, cv::COLOR_BGR2GRAY);
  cv::cvtColor(oriented, oriented_gray, cv::COLOR_BGR2GRAY);

  std::vector<cv::KeyPoint> processed_keypoints;
  std::vector<cv::KeyPoint> oriented_keypoints;
  cv::Mat processed_descriptors;
  cv::Mat oriented_descriptors;
  cv::Ptr<cv::ORB> orb = cv::ORB::create(1500);
  orb->detectAndCompute(processed_gray, cv::noArray(), processed_keypoints,
                        processed_descriptors);
  orb->detectAndCompute(oriented_gray, cv::noArray(), oriented_keypoints,
                        oriented_descriptors);
  if (processed_descriptors.empty() || oriented_descriptors.empty()) {
    estimate.message = "Estimated geometry unavailable: not enough visual features.";
    return estimate;
  }

  std::vector<cv::DMatch> matches;
  cv::BFMatcher matcher(cv::NORM_HAMMING, true);
  matcher.match(processed_descriptors, oriented_descriptors, matches);
  std::sort(matches.begin(), matches.end(),
            [](const cv::DMatch& a, const cv::DMatch& b) {
              return a.distance < b.distance;
            });
  if (matches.size() > 200) {
    matches.resize(200);
  }
  constexpr size_t kMinMatches = 30;
  if (matches.size() < kMinMatches) {
    std::ostringstream message;
    message << "Estimated geometry unavailable: only " << matches.size()
            << " feature matches; at least " << kMinMatches << " required.";
    estimate.message = message.str();
    return estimate;
  }

  const double inlier_threshold =
      std::max(8.0, static_cast<double>(
                        std::max(processed.cols, processed.rows)) * 0.01);
  std::mt19937 rng(12345);
  std::uniform_int_distribution<size_t> pick(0, matches.size() - 1);
  int best_inliers = 0;
  double best_median = std::numeric_limits<double>::max();
  cv::Mat best_homography;

  for (int iteration = 0; iteration < 384; ++iteration) {
    std::vector<size_t> sample_indices;
    while (sample_indices.size() < 4) {
      const size_t candidate = pick(rng);
      if (std::find(sample_indices.begin(), sample_indices.end(), candidate) ==
          sample_indices.end()) {
        sample_indices.push_back(candidate);
      }
    }

    std::vector<cv::Point2f> src;
    std::vector<cv::Point2f> dst;
    src.reserve(4);
    dst.reserve(4);
    for (const size_t index : sample_indices) {
      const cv::DMatch& match = matches[index];
      src.push_back(processed_keypoints[match.queryIdx].pt);
      dst.push_back(oriented_keypoints[match.trainIdx].pt);
    }

    cv::Mat homography;
    try {
      homography = cv::getPerspectiveTransform(src, dst);
    } catch (const cv::Exception&) {
      continue;
    }
    if (!homography_is_usable(homography)) {
      continue;
    }

    std::vector<double> inlier_errors;
    inlier_errors.reserve(matches.size());
    for (const auto& match : matches) {
      const cv::Point2f projected =
          project_point(homography, processed_keypoints[match.queryIdx].pt);
      if (!std::isfinite(projected.x) || !std::isfinite(projected.y)) {
        continue;
      }
      const double error = point_error(projected, oriented_keypoints[match.trainIdx].pt);
      if (error <= inlier_threshold) {
        inlier_errors.push_back(error);
      }
    }
    if (inlier_errors.empty()) {
      continue;
    }
    std::sort(inlier_errors.begin(), inlier_errors.end());
    const double median = inlier_errors[inlier_errors.size() / 2];
    const int inliers = static_cast<int>(inlier_errors.size());
    if (inliers > best_inliers ||
        (inliers == best_inliers && median < best_median)) {
      best_inliers = inliers;
      best_median = median;
      best_homography = homography;
    }
  }

  const double inlier_ratio =
      matches.empty() ? 0.0
                      : static_cast<double>(best_inliers) /
                            static_cast<double>(matches.size());
  if (best_inliers < 12 || inlier_ratio < 0.35 ||
      best_median > inlier_threshold || !homography_is_usable(best_homography)) {
    std::ostringstream message;
    message << "Estimated geometry unavailable: feature mapping failed gates"
            << " (matches=" << matches.size() << ", inliers=" << best_inliers
            << ", ratio=" << inlier_ratio << ", medianError=" << best_median
            << ").";
    estimate.message = message.str();
    return estimate;
  }

  estimate.available = true;
  estimate.geometry = AnnotationGeometry::kEstimated;
  estimate.confidence = std::min(1.0, inlier_ratio);
  estimate.processed_to_source = best_homography;
  std::ostringstream message;
  message << "Estimated from feature homography"
          << " (matches=" << matches.size() << ", inliers=" << best_inliers
          << ", medianError=" << best_median << ").";
  estimate.message = message.str();
  return estimate;
}

cv::Mat resize_for_detection(const cv::Mat& image, int limit_side_len,
                             const std::string& limit_type) {
  cv::Mat input = image;
  if (input.rows + input.cols < 64) {
    const int pad_h = std::max(32, input.rows);
    const int pad_w = std::max(32, input.cols);
    cv::Mat padded = cv::Mat::zeros(pad_h, pad_w, input.type());
    input.copyTo(padded(cv::Rect(0, 0, input.cols, input.rows)));
    input = padded;
  }

  const int h = input.rows;
  const int w = input.cols;
  float ratio = 1.0f;
  if (limit_type == "max") {
    if (std::max(h, w) > limit_side_len) {
      ratio = static_cast<float>(limit_side_len) / std::max(h, w);
    }
  } else if (limit_type == "min") {
    if (std::min(h, w) < limit_side_len) {
      ratio = static_cast<float>(limit_side_len) / std::min(h, w);
    }
  } else if (limit_type == "resize_long") {
    ratio = static_cast<float>(limit_side_len) / std::max(h, w);
  } else {
    throw std::invalid_argument("unsupported detection limit type: " +
                                limit_type);
  }

  int resize_h = static_cast<int>(h * ratio);
  int resize_w = static_cast<int>(w * ratio);
  constexpr int kMaxSideLimit = 4000;
  if (std::max(resize_h, resize_w) > kMaxSideLimit) {
    ratio = static_cast<float>(kMaxSideLimit) / std::max(resize_h, resize_w);
    resize_h = static_cast<int>(resize_h * ratio);
    resize_w = static_cast<int>(resize_w * ratio);
  }
  const int64_t detection_pixels =
      static_cast<int64_t>(resize_h) * static_cast<int64_t>(resize_w);
  if (detection_pixels > kMaxDetectionPixels) {
    const double pixel_ratio =
        std::sqrt(static_cast<double>(kMaxDetectionPixels) /
                  static_cast<double>(detection_pixels));
    resize_h = std::max(1, static_cast<int>(resize_h * pixel_ratio));
    resize_w = std::max(1, static_cast<int>(resize_w * pixel_ratio));
  }
  resize_h = std::max(static_cast<int>(std::round(resize_h / 32.0) * 32), 32);
  resize_w = std::max(static_cast<int>(std::round(resize_w / 32.0) * 32), 32);

  if (resize_h == h && resize_w == w) {
    return input.clone();
  }
  {
    std::ostringstream message;
    message << "core ppocr detection resize " << w << "x" << h << " -> "
            << resize_w << "x" << resize_h;
    core_log_info(message.str());
  }
  cv::Mat resized;
  cv::resize(input, resized, cv::Size(resize_w, resize_h));
  return resized;
}

int argmax(const Tensor& tensor) {
  if (tensor.data.empty()) {
    throw std::runtime_error("empty classifier output");
  }
  return static_cast<int>(std::distance(
      tensor.data.begin(),
      std::max_element(tensor.data.begin(), tensor.data.end())));
}

std::pair<std::vector<cv::Point2f>, float> get_mini_boxes(
    const std::vector<cv::Point2f>& contour) {
  cv::RotatedRect box = cv::minAreaRect(contour);
  std::vector<cv::Point2f> points(4);
  box.points(points.data());
  std::sort(points.begin(), points.end(),
            [](const cv::Point2f& a, const cv::Point2f& b) {
              return a.x < b.x;
            });

  int index_1 = 0;
  int index_2 = 1;
  int index_3 = 2;
  int index_4 = 3;
  if (points[1].y > points[0].y) {
    index_1 = 0;
    index_4 = 1;
  } else {
    index_1 = 1;
    index_4 = 0;
  }
  if (points[3].y > points[2].y) {
    index_2 = 2;
    index_3 = 3;
  } else {
    index_2 = 3;
    index_3 = 2;
  }

  return {{points[index_1], points[index_2], points[index_3],
           points[index_4]},
          std::min(box.size.width, box.size.height)};
}

float box_score_fast(const cv::Mat& pred, std::vector<cv::Point2f> contour) {
  if (contour.empty()) {
    return 0.0f;
  }
  int xmin = std::max(
      0, static_cast<int>(std::floor(
             std::min_element(contour.begin(), contour.end(),
                              [](const cv::Point2f& a, const cv::Point2f& b) {
                                return a.x < b.x;
                              })
                 ->x)));
  int xmax = std::max(
      0, static_cast<int>(std::ceil(
             std::max_element(contour.begin(), contour.end(),
                              [](const cv::Point2f& a, const cv::Point2f& b) {
                                return a.x < b.x;
                              })
                 ->x)));
  int ymin = std::max(
      0, static_cast<int>(std::floor(
             std::min_element(contour.begin(), contour.end(),
                              [](const cv::Point2f& a, const cv::Point2f& b) {
                                return a.y < b.y;
                              })
                 ->y)));
  int ymax = std::max(
      0, static_cast<int>(std::ceil(
             std::max_element(contour.begin(), contour.end(),
                              [](const cv::Point2f& a, const cv::Point2f& b) {
                                return a.y < b.y;
                              })
                 ->y)));

  xmin = std::min(xmin, pred.cols - 1);
  xmax = std::min(xmax, pred.cols - 1);
  ymin = std::min(ymin, pred.rows - 1);
  ymax = std::min(ymax, pred.rows - 1);
  if (xmax < xmin || ymax < ymin) {
    return 0.0f;
  }

  cv::Mat mask = cv::Mat::zeros(ymax - ymin + 1, xmax - xmin + 1, CV_8UC1);
  std::vector<cv::Point> contour_int;
  contour_int.reserve(contour.size());
  for (auto& point : contour) {
    contour_int.emplace_back(static_cast<int>(point.x - xmin),
                             static_cast<int>(point.y - ymin));
  }
  cv::fillPoly(mask, std::vector<std::vector<cv::Point>>{contour_int},
               cv::Scalar(1));
  const cv::Mat roi = pred(cv::Rect(xmin, ymin, xmax - xmin + 1,
                                    ymax - ymin + 1));
  return static_cast<float>(cv::mean(roi, mask)[0]);
}

std::vector<cv::Point2f> unclip(const std::vector<cv::Point2f>& box,
                                float unclip_ratio) {
  const float area = static_cast<float>(cv::contourArea(box));
  const float length = static_cast<float>(cv::arcLength(box, true));
  if (length <= std::numeric_limits<float>::epsilon()) {
    return {};
  }
  const float distance = area * unclip_ratio / length;

  ClipperLib::Path path;
  for (const auto& point : box) {
    path.emplace_back(static_cast<ClipperLib::cInt>(point.x),
                      static_cast<ClipperLib::cInt>(point.y));
  }
  ClipperLib::ClipperOffset offset;
  offset.AddPath(path, ClipperLib::jtRound, ClipperLib::etClosedPolygon);

  ClipperLib::Paths solution;
  offset.Execute(solution, distance);
  if (solution.empty()) {
    return {};
  }
  std::vector<cv::Point2f> result;
  result.reserve(solution[0].size());
  for (const auto& point : solution[0]) {
    result.emplace_back(static_cast<float>(point.X),
                        static_cast<float>(point.Y));
  }
  return result;
}

std::vector<std::vector<cv::Point2f>> db_postprocess(const Tensor& output,
                                                     const cv::Size& image_size,
                                                     const RunOptions& options) {
  if (output.shape.size() != 4 || output.shape[0] != 1 ||
      output.shape[1] != 1) {
    throw std::runtime_error("unexpected detection output shape");
  }
  const int pred_h = static_cast<int>(output.shape[2]);
  const int pred_w = static_cast<int>(output.shape[3]);
  cv::Mat pred(pred_h, pred_w, CV_32F, const_cast<float*>(output.data.data()));
  cv::Mat bitmap = pred > options.text_detection_threshold;

  std::vector<std::vector<cv::Point>> contours_int;
  cv::findContours(bitmap, contours_int, cv::RETR_LIST, cv::CHAIN_APPROX_SIMPLE);
  const int num_contours =
      std::min(static_cast<int>(contours_int.size()), 3000);
  std::vector<std::vector<cv::Point2f>> boxes;
  boxes.reserve(num_contours);

  const float width_scale =
      static_cast<float>(image_size.width) / static_cast<float>(pred_w);
  const float height_scale =
      static_cast<float>(image_size.height) / static_cast<float>(pred_h);
  constexpr float kMinSize = 3.0f;

  for (int i = 0; i < num_contours; ++i) {
    std::vector<cv::Point2f> contour;
    contour.reserve(contours_int[i].size());
    for (const auto& point : contours_int[i]) {
      contour.emplace_back(static_cast<float>(point.x),
                           static_cast<float>(point.y));
    }
    if (contour.size() < 4) {
      continue;
    }
    auto mini = get_mini_boxes(contour);
    if (mini.second < kMinSize) {
      continue;
    }

    const float score = box_score_fast(pred, mini.first);
    if (score < options.text_detection_box_threshold) {
      continue;
    }

    std::vector<cv::Point2f> unclipped =
        unclip(mini.first, options.text_detection_unclip_ratio);
    if (unclipped.empty()) {
      continue;
    }
    auto expanded = get_mini_boxes(unclipped);
    if (expanded.second < kMinSize + 2.0f) {
      continue;
    }

    for (auto& point : expanded.first) {
      point.x = static_cast<float>(std::max(
          0, std::min(static_cast<int>(std::round(point.x * width_scale)),
                      image_size.width - 1)));
      point.y = static_cast<float>(std::max(
          0, std::min(static_cast<int>(std::round(point.y * height_scale)),
                      image_size.height - 1)));
    }
    boxes.push_back(expanded.first);
  }
  return boxes;
}

std::vector<std::vector<cv::Point2f>> sort_quad_boxes(
    std::vector<std::vector<cv::Point2f>> boxes) {
  if (boxes.empty()) {
    return boxes;
  }
  std::sort(boxes.begin(), boxes.end(),
            [](const std::vector<cv::Point2f>& a,
               const std::vector<cv::Point2f>& b) {
              return (a[0].y < b[0].y) ||
                     (a[0].y == b[0].y && a[0].x < b[0].x);
            });
  for (size_t i = 0; i + 1 < boxes.size(); ++i) {
    for (size_t j = i + 1; j > 0; --j) {
      if (std::abs(boxes[j][0].y - boxes[j - 1][0].y) < 10 &&
          boxes[j][0].x < boxes[j - 1][0].x) {
        std::swap(boxes[j], boxes[j - 1]);
      } else {
        break;
      }
    }
  }
  return boxes;
}

cv::Mat crop_quad(const cv::Mat& image, const std::vector<cv::Point2f>& box) {
  if (box.size() != 4) {
    throw std::invalid_argument("quad crop expects exactly four points");
  }
  const float width_top = static_cast<float>(cv::norm(box[0] - box[1]));
  const float width_bottom = static_cast<float>(cv::norm(box[2] - box[3]));
  const float max_width = std::max(width_top, width_bottom);
  const float height_left = static_cast<float>(cv::norm(box[0] - box[3]));
  const float height_right = static_cast<float>(cv::norm(box[1] - box[2]));
  const float max_height = std::max(height_left, height_right);
  if (max_width < 1 || max_height < 1) {
    throw std::invalid_argument("quad crop is too small");
  }
  std::vector<cv::Point2f> dst = {
      {0.0f, 0.0f},
      {max_width - 1.0f, 0.0f},
      {max_width - 1.0f, max_height - 1.0f},
      {0.0f, max_height - 1.0f},
  };
  cv::Mat transform = cv::getPerspectiveTransform(box, dst);
  cv::Mat out;
  cv::warpPerspective(image, out, transform,
                      cv::Size(static_cast<int>(max_width),
                               static_cast<int>(max_height)),
                      cv::INTER_CUBIC, cv::BORDER_REPLICATE);
  if (out.rows != 0 && out.cols != 0 &&
      static_cast<float>(out.rows) / static_cast<float>(out.cols) >= 1.5f) {
    cv::rotate(out, out, cv::ROTATE_90_COUNTERCLOCKWISE);
  }
  return out;
}

std::vector<float> recognition_input(const cv::Mat& bgr,
                                     std::vector<int64_t>* shape) {
  constexpr int kRecC = 3;
  constexpr int kRecH = 48;
  constexpr int kBaseRecW = 320;
  constexpr int kMaxRecW = 3200;
  const float rec_wh_ratio = static_cast<float>(kBaseRecW) / kRecH;
  const float image_wh_ratio =
      static_cast<float>(bgr.cols) / static_cast<float>(bgr.rows);
  const float max_wh_ratio = std::max(rec_wh_ratio, image_wh_ratio);
  int rec_w = static_cast<int>(kRecH * max_wh_ratio);
  rec_w = std::max(kBaseRecW, std::min(kMaxRecW, rec_w));

  int resize_w = 0;
  if (rec_w >= kMaxRecW) {
    resize_w = kMaxRecW;
  } else {
    resize_w = std::min(rec_w,
                        static_cast<int>(std::ceil(kRecH * image_wh_ratio)));
  }
  resize_w = std::max(1, resize_w);

  cv::Mat resized;
  cv::resize(bgr, resized, cv::Size(resize_w, kRecH));

  std::vector<float> out(static_cast<size_t>(kRecC * kRecH * rec_w), 0.0f);
  for (int y = 0; y < resized.rows; ++y) {
    const cv::Vec3b* row = resized.ptr<cv::Vec3b>(y);
    for (int x = 0; x < resized.cols; ++x) {
      const cv::Vec3b pixel = row[x];
      const int offset = y * rec_w + x;
      for (int c = 0; c < kRecC; ++c) {
        out[static_cast<size_t>(c * kRecH * rec_w + offset)] =
            (static_cast<float>(pixel[c]) / 255.0f - 0.5f) / 0.5f;
      }
    }
  }
  *shape = {1, kRecC, kRecH, rec_w};
  return out;
}

std::pair<std::string, float> ctc_decode(
    const Tensor& tensor, const std::vector<std::string>& characters) {
  if (tensor.shape.size() != 3 || tensor.shape[0] != 1) {
    throw std::runtime_error("unexpected recognition output shape");
  }
  const int seq_len = static_cast<int>(tensor.shape[1]);
  const int classes = static_cast<int>(tensor.shape[2]);
  std::string text;
  std::vector<float> selected_probs;
  int previous = -1;
  for (int t = 0; t < seq_len; ++t) {
    const float* row =
        tensor.data.data() + static_cast<size_t>(t) * classes;
    int max_idx = 0;
    float max_value = row[0];
    for (int c = 1; c < classes; ++c) {
      if (row[c] > max_value) {
        max_value = row[c];
        max_idx = c;
      }
    }
    const bool duplicate = max_idx == previous;
    previous = max_idx;
    if (max_idx == 0 || duplicate) {
      continue;
    }
    if (max_idx >= 0 && static_cast<size_t>(max_idx) < characters.size()) {
      text += characters[static_cast<size_t>(max_idx)];
    } else {
      text += " ";
    }
    selected_probs.push_back(max_value);
  }
  if (selected_probs.empty()) {
    return {"", 0.0f};
  }
  const float sum =
      std::accumulate(selected_probs.begin(), selected_probs.end(), 0.0f);
  return {text, sum / static_cast<float>(selected_probs.size())};
}

class NativeEngine {
 public:
  NativeEngine(const std::string& model_root, const RuntimeOptions& runtime)
      : runtime_(runtime),
        active_backend_(resolve_backend(runtime)),
        doc_orientation_(join_path(model_root, "doc_orientation/inference.onnx"),
                         runtime, "doc_orientation", active_backend_),
        doc_unwarping_(join_path(model_root, "doc_unwarping/inference.onnx"),
                       runtime, "doc_unwarping", active_backend_),
        textline_orientation_(
            join_path(model_root, "textline_orientation/inference.onnx"),
            runtime, "textline_orientation", active_backend_),
        text_detection_(join_path(model_root, "text_detection/inference.onnx"),
                        runtime, "text_detection", active_backend_),
        text_recognition_(
            join_path(model_root, "text_recognition/inference.onnx"), runtime,
            "text_recognition", active_backend_),
        characters_(load_character_dict(
            join_path(model_root, "text_recognition/inference.yml"))) {}

  std::string runtime_summary() const {
    return make_runtime_summary(active_backend_, runtime_.cpu_threads);
  }

  agus_ocr_backend_t active_backend() const { return active_backend_; }

  std::string recognize(const agus_ocr_image_t& image,
                        const RunOptions& options,
                        std::vector<std::string> initial_warnings = {}) const {
    const auto total_start = std::chrono::steady_clock::now();
    Timing timing;
    std::vector<std::string> warnings = std::move(initial_warnings);
    {
      std::ostringstream message;
      message << "core ppocr recognize start bytes=" << image.length
              << " maxPixels=" << options.max_pixels;
      core_log_info(message.str());
    }
    cv::Mat page = decode_image(image);
    {
      std::ostringstream message;
      message << "core ppocr decode complete width=" << page.cols
              << " height=" << page.rows
              << " pixels="
              << static_cast<int64_t>(page.cols) * page.rows;
      core_log_info(message.str());
    }
    page = normalize_input_image(page, options, &warnings);
    const cv::Mat source_page = page.clone();
    cv::Mat source_to_oriented =
        (cv::Mat_<double>(2, 3) << 1.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    cv::Mat oriented_page = page.clone();
    bool used_doc_unwarping = false;
    int doc_angle = -1;

    if (options.use_doc_orientation) {
      const auto start = std::chrono::steady_clock::now();
      core_log_info("core ppocr doc orientation start");
      cv::Mat resized = resize_short_and_crop(page, 256, 224);
      const auto input = normalize_to_chw(
          resized, true, {0.485f, 0.456f, 0.406f},
          {0.229f, 0.224f, 0.225f}, 1.0f / 255.0f);
      const Tensor output = doc_orientation_.run(input, {1, 3, 224, 224});
      const int class_id = argmax(output);
      static constexpr int kDocAngles[4] = {0, 90, 180, 270};
      if (class_id >= 0 && class_id < 4) {
        doc_angle = kDocAngles[class_id];
        page = rotate_image_with_transform(page, doc_angle, &source_to_oriented);
        oriented_page = page.clone();
      }
      timing.doc_orientation_ms =
          elapsed_ms(start, std::chrono::steady_clock::now());
      {
        std::ostringstream message;
        message << "core ppocr doc orientation complete angle=" << doc_angle
                << " elapsedMs=" << timing.doc_orientation_ms;
        core_log_info(message.str());
      }
    }
    if (!options.use_doc_orientation || doc_angle < 0) {
      oriented_page = page.clone();
    }

    if (options.use_doc_unwarping) {
      const auto start = std::chrono::steady_clock::now();
      core_log_info("core ppocr doc unwarping start");
      const auto input = scale_bgr_to_chw(page, 1.0f / 255.0f);
      const Tensor output =
          doc_unwarping_.run(input, {1, 3, page.rows, page.cols});
      if (output.shape.size() != 4 || output.shape[0] != 1 ||
          output.shape[1] != 3) {
        throw std::runtime_error("unexpected document unwarping output shape");
      }
      const int h = static_cast<int>(output.shape[2]);
      const int w = static_cast<int>(output.shape[3]);
      cv::Mat unwarped(h, w, CV_8UC3);
      const int plane = h * w;
      for (int y = 0; y < h; ++y) {
        cv::Vec3b* row = unwarped.ptr<cv::Vec3b>(y);
        for (int x = 0; x < w; ++x) {
          const int offset = y * w + x;
          for (int c = 0; c < 3; ++c) {
            const float value =
                output.data[static_cast<size_t>(c * plane + offset)] * 255.0f;
            row[x][c] = static_cast<uchar>(std::max(
                0.0f, std::min(255.0f, std::round(value))));
          }
        }
      }
      page = unwarped;
      used_doc_unwarping = true;
      timing.doc_unwarping_ms =
          elapsed_ms(start, std::chrono::steady_clock::now());
      {
        std::ostringstream message;
        message << "core ppocr doc unwarping complete width=" << page.cols
                << " height=" << page.rows
                << " elapsedMs=" << timing.doc_unwarping_ms;
        core_log_info(message.str());
      }
    }

    std::vector<std::vector<cv::Point2f>> boxes;
    {
      const auto start = std::chrono::steady_clock::now();
      core_log_info("core ppocr detection start");
      cv::Mat det_image = resize_for_detection(
          page, options.text_detection_limit_side_len,
          options.text_detection_limit_type);
      {
        std::ostringstream message;
        message << "core ppocr detection input width=" << det_image.cols
                << " height=" << det_image.rows
                << " pixels="
                << static_cast<int64_t>(det_image.cols) * det_image.rows;
        core_log_info(message.str());
      }
      const auto input = normalize_to_chw(
          det_image, true, {0.485f, 0.456f, 0.406f},
          {0.229f, 0.224f, 0.225f}, 1.0f / 255.0f);
      const Tensor output =
          text_detection_.run(input, {1, 3, det_image.rows, det_image.cols});
      boxes = sort_quad_boxes(db_postprocess(output, page.size(), options));
      timing.detection_ms = elapsed_ms(start, std::chrono::steady_clock::now());
      {
        std::ostringstream message;
        message << "core ppocr detection complete boxes=" << boxes.size()
                << " elapsedMs=" << timing.detection_ms;
        core_log_info(message.str());
      }
    }

    std::vector<OcrLine> lines;
    lines.reserve(boxes.size());
    {
      std::ostringstream message;
      message << "core ppocr recognition loop start boxes=" << boxes.size();
      core_log_info(message.str());
    }
    for (const auto& box : boxes) {
      cv::Mat crop;
      try {
        crop = crop_quad(page, box);
      } catch (const std::exception&) {
        continue;
      }

      int textline_angle = -1;
      if (options.use_textline_orientation) {
        const auto start = std::chrono::steady_clock::now();
        cv::Mat resized;
        cv::resize(crop, resized, cv::Size(160, 80), 0, 0, cv::INTER_LINEAR);
        const auto input = normalize_to_chw(
            resized, true, {0.485f, 0.456f, 0.406f},
            {0.229f, 0.224f, 0.225f}, 1.0f / 255.0f);
        const Tensor output =
            textline_orientation_.run(input, {1, 3, 80, 160});
        const int class_id = argmax(output);
        textline_angle = class_id == 1 ? 180 : 0;
        if (textline_angle == 180) {
          crop = rotate_image(crop, 180);
        }
        timing.textline_orientation_ms +=
            elapsed_ms(start, std::chrono::steady_clock::now());
      }

      const auto start = std::chrono::steady_clock::now();
      std::vector<int64_t> shape;
      const auto input = recognition_input(crop, &shape);
      const Tensor output = text_recognition_.run(input, shape);
      const auto rec = ctc_decode(output, characters_);
      timing.recognition_ms +=
          elapsed_ms(start, std::chrono::steady_clock::now());

      if (rec.second < options.text_recognition_score_threshold) {
        continue;
      }
      OcrLine line;
      line.text = rec.first;
      line.confidence = rec.second;
      line.textline_angle = textline_angle;
      line.polygon.reserve(box.size());
      for (const auto& point : box) {
        line.polygon.push_back({point.x, point.y});
      }
      lines.push_back(std::move(line));
    }
    {
      std::ostringstream message;
      message << "core ppocr recognition loop complete lines=" << lines.size()
              << " elapsedMs=" << timing.recognition_ms;
      core_log_info(message.str());
    }

    timing.total_ms = elapsed_ms(total_start, std::chrono::steady_clock::now());
    core_log_info("core ppocr result json build start");
    std::string json =
        build_json(source_page, oriented_page, page, doc_angle, lines, timing,
                   source_to_oriented, used_doc_unwarping, options, warnings);
    {
      std::ostringstream message;
      message << "core ppocr result json build complete bytes=" << json.size();
      core_log_info(message.str());
    }
    return json;
  }

 private:
  std::string build_json(const cv::Mat& source_page,
                         const cv::Mat& oriented_page,
                         const cv::Mat& page,
                         int doc_angle,
                         const std::vector<OcrLine>& lines,
                         const Timing& timing,
                         const cv::Mat& source_to_oriented,
                         bool used_doc_unwarping,
                         const RunOptions& options,
                         const std::vector<std::string>& warnings) const {
    std::ostringstream text;
    for (size_t i = 0; i < lines.size(); ++i) {
      if (i > 0) {
        text << '\n';
      }
      text << lines[i].text;
    }
    const std::string page_text = text.str();
    std::string overlay_mime_type;
    const std::string overlay_base64 =
        encode_overlay_image_base64(page, &overlay_mime_type);

    cv::Mat oriented_to_source_affine;
    cv::invertAffineTransform(source_to_oriented, oriented_to_source_affine);
    const cv::Mat oriented_to_source =
        affine_to_homography(oriented_to_source_affine);

    MappingEstimate source_estimate;
    if (!options.enable_source_box_estimation) {
      source_estimate.available = false;
      source_estimate.geometry = AnnotationGeometry::kNone;
      source_estimate.confidence = 0.0;
      source_estimate.message =
          "Source-box estimation is disabled by run options.";
    } else if (!used_doc_unwarping) {
      source_estimate.available = true;
      source_estimate.geometry = AnnotationGeometry::kExact;
      source_estimate.confidence = 1.0;
      source_estimate.processed_to_source = oriented_to_source;
      source_estimate.message =
          "Exact source geometry from inverse orientation transform.";
    } else {
      source_estimate = estimate_processed_to_oriented(page, oriented_page);
      if (source_estimate.available) {
        source_estimate.processed_to_source =
            oriented_to_source * source_estimate.processed_to_source;
      }
    }

    std::vector<OcrLine> source_estimated_lines;
    if (source_estimate.available) {
      source_estimated_lines =
          transform_lines(lines, source_estimate.processed_to_source,
                          source_page.size());
    }

    std::ostringstream out;
    out << "{\"status\":0,\"message\":\"ok\",\"pages\":[{\"pageIndex\":0,"
        << "\"width\":" << page.cols << ",\"height\":" << page.rows
        << ",\"overlayImageMimeType\":\"" << overlay_mime_type
        << "\",\"overlayImageBytesBase64\":\"" << overlay_base64
        << "\",\"docAngle\":" << doc_angle << ",\"lines\":";
    append_lines_json(&out, lines);
    out << ",\"annotationLayers\":[";
    append_annotation_layer_json(
        &out, "source", "Original", source_page, std::vector<OcrLine>{},
        AnnotationGeometry::kNone, true, 1.0,
        "Original native-decoded image. No canonical source boxes are available.");
    out << ',';
    append_annotation_layer_json(&out, "processed", "Processed", page, lines,
                                 AnnotationGeometry::kExact, true, 1.0,
                                 "Exact OCR geometry in processed image space.");
    out << ',';
    append_annotation_layer_json(
        &out, "source_estimated", "Estimated", source_page,
        source_estimated_lines, source_estimate.geometry,
        source_estimate.available, source_estimate.confidence,
        source_estimate.message);
    const std::string markdown_text =
        options.generate_markdown ? markdown_from_lines(lines) : "";
    out << "],\"text\":\"" << json_escape(page_text)
        << "\",\"markdownText\":\"" << json_escape(markdown_text)
        << "\",\"structuredJson\":\"{}\",\"blocks\":";
    append_blocks_json(&out, lines, "processed");
    out << "}"
        << "],\"text\":\"" << json_escape(page_text)
        << "\",\"markdownText\":\"" << json_escape(markdown_text)
        << "\",\"structuredJson\":\"{}"
        << "\",\"timing\":{\"docOrientationMs\":" << timing.doc_orientation_ms
        << ",\"docUnwarpingMs\":" << timing.doc_unwarping_ms
        << ",\"detectionMs\":" << timing.detection_ms
        << ",\"textLineOrientationMs\":" << timing.textline_orientation_ms
        << ",\"recognitionMs\":" << timing.recognition_ms
        << ",\"totalMs\":" << timing.total_ms << "},\"modelSummary\":\""
        << kModelSummary << "\",\"runtimeSummary\":\""
        << json_escape(runtime_summary()) << "\",\"warnings\":";
    append_string_array_json(&out, warnings);
    out << "}";
    return out.str();
  }

  RuntimeOptions runtime_;
  agus_ocr_backend_t active_backend_ = AGUS_OCR_BACKEND_CPU;
  OrtRunner doc_orientation_;
  OrtRunner doc_unwarping_;
  OrtRunner textline_orientation_;
  OrtRunner text_detection_;
  OrtRunner text_recognition_;
  std::vector<std::string> characters_;
};

#endif  // AGUS_OCR_ENABLE_NATIVE_PIPELINE

}  // namespace

struct agus_ocr_engine_t {
  agus_ocr_pipeline_t pipeline = AGUS_OCR_PIPELINE_PPOCRV6;
  std::string model_root;
  std::string external_model_root;
  std::string vl_model_path;
  std::string vl_mmproj_path;
  RuntimeOptions requested_runtime;
  RuntimeOptions runtime;
  RunOptions defaults;
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
  std::unique_ptr<NativeEngine> native;
  agus_ocr_run_options_t vl_defaults{};
  std::unique_ptr<agus_ocr::PaddleOcrVlEngine> vl_native;
  agus_ocr_run_options_t gemma_defaults{};
  std::unique_ptr<agus_ocr::GemmaMarkdownEngine> gemma_native;
#endif
};

#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
bool is_directml_failure_message(const std::string& message) {
  std::string lower = lower_ascii(message);
  return lower.find("directml") != std::string::npos ||
         lower.find("dmlexecutionprovider") != std::string::npos ||
         lower.find("mloperator") != std::string::npos ||
         lower.find("d3d12") != std::string::npos ||
         lower.find("dxgi") != std::string::npos ||
         lower.find("appendexecutionprovider_dml") != std::string::npos;
}

bool is_ort_or_backend_failure_message(agus_ocr_backend_t backend,
                                       const std::string& message) {
  if (backend == AGUS_OCR_BACKEND_DIRECTML) {
    return is_directml_failure_message(message);
  }
  const std::string lower = lower_ascii(message);
  return lower.find("onnxruntime") != std::string::npos ||
         lower.find("executionprovider") != std::string::npos ||
         lower.find("execution provider") != std::string::npos ||
         lower.find("non-zero status code") != std::string::npos ||
         lower.find("session") != std::string::npos ||
         lower.find("ort") != std::string::npos ||
         lower.find(backend_label(backend)) != std::string::npos;
}

bool runtime_allows_onnx_backend_fallback(const RuntimeOptions& runtime) {
  return !runtime.force_cpu_only && runtime.backend != AGUS_OCR_BACKEND_CPU;
}

bool should_retry_ppocr_create(const RuntimeOptions& runtime,
                               agus_ocr_backend_t failed_backend,
                               const std::string& message) {
  return runtime_allows_onnx_backend_fallback(runtime) &&
         failed_backend != AGUS_OCR_BACKEND_CPU &&
         is_ort_or_backend_failure_message(failed_backend, message);
}

bool should_retry_ppocr_recognition(const agus_ocr_engine_t& engine,
                                    const std::string& message) {
  if (!runtime_allows_onnx_backend_fallback(engine.requested_runtime) ||
      !engine.native ||
      engine.native->active_backend() == AGUS_OCR_BACKEND_CPU) {
    return false;
  }
  return is_ort_or_backend_failure_message(engine.native->active_backend(),
                                           message);
}

std::vector<agus_ocr_backend_t> remaining_ppocr_attempts_after_failure(
    const RuntimeOptions& runtime, agus_ocr_backend_t failed_backend) {
  if (!runtime_allows_onnx_backend_fallback(runtime)) {
    return {};
  }
#if defined(__ANDROID__)
  const std::vector<agus_ocr_backend_t> priority = {
      AGUS_OCR_BACKEND_QNN, AGUS_OCR_BACKEND_NNAPI,
      AGUS_OCR_BACKEND_XNNPACK, AGUS_OCR_BACKEND_CPU};
#else
  const std::vector<agus_ocr_backend_t> priority = {
      AGUS_OCR_BACKEND_DIRECTML, AGUS_OCR_BACKEND_CPU};
#endif
  if (runtime.backend != AGUS_OCR_BACKEND_AUTO) {
    return failed_backend == AGUS_OCR_BACKEND_CPU
               ? std::vector<agus_ocr_backend_t>{}
               : std::vector<agus_ocr_backend_t>{AGUS_OCR_BACKEND_CPU};
  }
  const auto failed = std::find(priority.begin(), priority.end(),
                                failed_backend);
  if (failed == priority.end()) {
    return {AGUS_OCR_BACKEND_CPU};
  }
  std::vector<agus_ocr_backend_t> out;
  for (auto it = failed + 1; it != priority.end(); ++it) {
    if (*it == AGUS_OCR_BACKEND_CPU || backend_supported_for_ppocr(*it)) {
      out.push_back(*it);
    }
  }
  return out;
}

std::vector<agus_ocr_backend_t> remaining_gemma_attempts_after_failure(
    const RuntimeOptions& runtime, agus_ocr_backend_t failed_backend) {
  if (!runtime_allows_onnx_backend_fallback(runtime)) {
    return {};
  }
  if (runtime.backend != AGUS_OCR_BACKEND_AUTO) {
    return failed_backend == AGUS_OCR_BACKEND_CPU
               ? std::vector<agus_ocr_backend_t>{}
               : std::vector<agus_ocr_backend_t>{AGUS_OCR_BACKEND_CPU};
  }
  const std::vector<agus_ocr_backend_t> priority = {
      AGUS_OCR_BACKEND_CUDA, AGUS_OCR_BACKEND_DIRECTML,
      AGUS_OCR_BACKEND_CPU};
  const auto failed = std::find(priority.begin(), priority.end(),
                                failed_backend);
  if (failed == priority.end()) {
    return {AGUS_OCR_BACKEND_CPU};
  }
  std::vector<agus_ocr_backend_t> out;
  for (auto it = failed + 1; it != priority.end(); ++it) {
    if (*it == AGUS_OCR_BACKEND_CPU || backend_supported_for_gemma(*it)) {
      out.push_back(*it);
    }
  }
  return out;
}

RuntimeOptions runtime_for_backend(const RuntimeOptions& requested,
                                   agus_ocr_backend_t backend) {
  RuntimeOptions runtime = requested;
  runtime.backend = backend;
  if (backend == AGUS_OCR_BACKEND_CPU) {
    runtime.force_cpu_only = true;
    runtime.generative_backend = AGUS_OCR_GEN_BACKEND_CPU;
    runtime.generative_gpu_layers = 0;
  }
  return runtime;
}

std::string ppocr_fallback_warning(agus_ocr_backend_t failed_backend,
                                   agus_ocr_backend_t retry_backend,
                                   const std::string& message) {
  return backend_display_name(failed_backend) +
         " failed during OCR and this run was retried on " +
         backend_display_name(retry_backend) + ". " +
         backend_display_name(failed_backend) +
         " is disabled for the rest of this process. Original error: " +
         message;
}
#endif

extern "C" {

agus_ocr_status_t agus_ocr_get_runtime_capabilities(
    agus_ocr_result_t** out_result) {
  if (out_result == nullptr) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "out_result is null");
  }
  *out_result = nullptr;
  try {
    RuntimeCapabilities capabilities = detect_runtime_capabilities();
    capabilities.runtime_summary =
        make_runtime_summary(capabilities.default_backend, 0);
    *out_result = allocate_result(capabilities_json(capabilities));
    g_last_error.clear();
    return AGUS_OCR_OK;
  } catch (const std::exception& error) {
    return fail(AGUS_OCR_INTERNAL_ERROR, error.what());
  }
}

agus_ocr_status_t agus_ocr_create(const agus_ocr_init_options_t* options,
                                  agus_ocr_engine_t** out_engine) {
  if (out_engine == nullptr) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "out_engine is null");
  }
  *out_engine = nullptr;
  if (options == nullptr ||
      !has_size(options->struct_size, sizeof(agus_ocr_init_options_t))) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "invalid init options");
  }
  if (options->model_root == nullptr || options->model_root[0] == '\0') {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "model_root is required");
  }
  if (!has_size(options->runtime.struct_size, kRequiredRuntimeOptionsSize) ||
      !has_size(options->defaults.struct_size, kRequiredRunOptionsSize)) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "invalid nested init options");
  }

  try {
    const agus_ocr_pipeline_t pipeline = options->pipeline;
    if (pipeline == AGUS_OCR_PIPELINE_GEMMA_MARKDOWN) {
      const std::string bundle_root =
          options->external_model_root != nullptr &&
                  options->external_model_root[0] != '\0'
              ? options->external_model_root
              : options->model_root;
      const agus_ocr::GemmaMarkdownBundleCheck bundle =
          agus_ocr::ValidateGemmaMarkdownBundle(bundle_root);
      if (!bundle.ok) {
        return fail(AGUS_OCR_UNAVAILABLE, bundle.message());
      }
#if defined(_WIN32) && defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
      auto engine = std::make_unique<agus_ocr_engine_t>();
      engine->pipeline = pipeline;
      engine->model_root = options->model_root;
      engine->external_model_root =
          options->external_model_root == nullptr ? "" : options->external_model_root;
      engine->requested_runtime = runtime_options_from_c(options->runtime);
      engine->runtime = engine->requested_runtime;
      engine->defaults = run_options_from_c(options->defaults);
      engine->gemma_defaults = run_options_to_c(engine->defaults);
      const std::vector<agus_ocr_backend_t> attempts =
          gemma_backend_attempts(engine->requested_runtime);
      std::vector<std::string> failures;
      for (agus_ocr_backend_t backend : attempts) {
        RuntimeOptions attempt_runtime =
            runtime_for_backend(engine->requested_runtime, backend);
        try {
          engine->gemma_native =
              std::make_unique<agus_ocr::GemmaMarkdownEngine>(
                  bundle, runtime_options_to_c(attempt_runtime), backend);
          engine->runtime = attempt_runtime;
          break;
        } catch (const std::exception& error) {
          const std::string failure = error.what();
          failures.push_back(backend_display_name(backend) + ": " + failure);
          if (!should_retry_ppocr_create(engine->requested_runtime, backend,
                                         failure)) {
            if (backend != AGUS_OCR_BACKEND_CPU) {
              mark_gemma_backend_unhealthy(backend, failure);
            }
            throw;
          }
          mark_gemma_backend_unhealthy(backend, failure);
        }
      }
      if (!engine->gemma_native) {
        std::ostringstream message;
        message << "Gemma Markdown initialization failed for all runtime "
                   "backends.";
        if (!failures.empty()) {
          message << " Attempts: ";
          for (size_t i = 0; i < failures.size(); ++i) {
            if (i > 0) {
              message << " | ";
            }
            message << failures[i];
          }
        }
        return fail(AGUS_OCR_INTERNAL_ERROR, message.str());
      }
      *out_engine = engine.release();
      g_last_error.clear();
      return AGUS_OCR_OK;
#else
      return fail(AGUS_OCR_UNAVAILABLE,
                  "Gemma Markdown ONNX pipeline is Windows-only in this build.");
#endif
    }

    if (pipeline == AGUS_OCR_PIPELINE_PADDLEOCR_VL16) {
      const std::string bundle_root =
          options->external_model_root != nullptr &&
                  options->external_model_root[0] != '\0'
              ? options->external_model_root
              : options->model_root;
      const agus_ocr::PaddleOcrVlBundleCheck bundle =
          agus_ocr::ValidatePaddleOcrVl16Bundle(
              bundle_root,
              options->vl_model_path == nullptr ? "" : options->vl_model_path,
              options->vl_mmproj_path == nullptr ? "" : options->vl_mmproj_path);
      if (!bundle.ok) {
        return fail(AGUS_OCR_UNAVAILABLE, bundle.message());
      }
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
      if (!agus_ocr::LlamaVlmRuntimeAvailable()) {
        return fail(AGUS_OCR_UNAVAILABLE,
                    agus_ocr::LlamaVlmUnavailableReason());
      }
      auto engine = std::make_unique<agus_ocr_engine_t>();
      engine->pipeline = pipeline;
      engine->model_root = options->model_root;
      engine->external_model_root =
          options->external_model_root == nullptr ? "" : options->external_model_root;
      engine->vl_model_path =
          options->vl_model_path == nullptr ? "" : options->vl_model_path;
      engine->vl_mmproj_path =
          options->vl_mmproj_path == nullptr ? "" : options->vl_mmproj_path;
      engine->requested_runtime = runtime_options_from_c(options->runtime);
      engine->runtime = engine->requested_runtime;
      engine->defaults = run_options_from_c(options->defaults);
      engine->vl_defaults = run_options_to_c(engine->defaults);
      const agus_ocr_runtime_options_t sanitized_runtime =
          runtime_options_to_c(engine->runtime);
      engine->vl_native =
          std::make_unique<agus_ocr::PaddleOcrVlEngine>(bundle,
                                                        sanitized_runtime);
      *out_engine = engine.release();
      g_last_error.clear();
      return AGUS_OCR_OK;
#else
      return fail(AGUS_OCR_UNAVAILABLE,
                  "native OCR core was built without ONNX Runtime/OpenCV support");
#endif
    }

    auto engine = std::make_unique<agus_ocr_engine_t>();
    engine->pipeline = pipeline;
    engine->model_root = options->model_root;
    engine->external_model_root =
        options->external_model_root == nullptr ? "" : options->external_model_root;
    engine->vl_model_path =
        options->vl_model_path == nullptr ? "" : options->vl_model_path;
    engine->vl_mmproj_path =
        options->vl_mmproj_path == nullptr ? "" : options->vl_mmproj_path;
    engine->requested_runtime = runtime_options_from_c(options->runtime);
    engine->runtime = engine->requested_runtime;
    engine->defaults = run_options_from_c(options->defaults);
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
    const std::vector<agus_ocr_backend_t> attempts =
        ppocr_backend_attempts(engine->requested_runtime);
    std::vector<std::string> failures;
    for (agus_ocr_backend_t backend : attempts) {
      RuntimeOptions attempt_runtime =
          runtime_for_backend(engine->requested_runtime, backend);
      try {
        engine->native =
            std::make_unique<NativeEngine>(engine->model_root, attempt_runtime);
        engine->runtime = attempt_runtime;
        break;
      } catch (const std::exception& error) {
        const std::string failure = error.what();
        failures.push_back(backend_display_name(backend) + ": " + failure);
        if (!should_retry_ppocr_create(engine->requested_runtime, backend,
                                       failure)) {
          if (backend != AGUS_OCR_BACKEND_CPU) {
            mark_backend_unhealthy(backend, failure);
          }
          throw;
        }
        mark_backend_unhealthy(backend, failure);
      }
    }
    if (!engine->native) {
      std::ostringstream message;
      message << "PP-OCRv6 initialization failed for all runtime backends.";
      if (!failures.empty()) {
        message << " Attempts: ";
        for (size_t i = 0; i < failures.size(); ++i) {
          if (i > 0) {
            message << " | ";
          }
          message << failures[i];
        }
      }
      return fail(AGUS_OCR_INTERNAL_ERROR, message.str());
    }
#endif
    *out_engine = engine.release();
    g_last_error.clear();
    return AGUS_OCR_OK;
  } catch (const std::exception& error) {
    return fail(AGUS_OCR_INTERNAL_ERROR, error.what());
  }
}

agus_ocr_status_t agus_ocr_recognize_image(agus_ocr_engine_t* engine,
                                           const agus_ocr_image_t* image,
                                           const agus_ocr_run_options_t* options,
                                           agus_ocr_result_t** out_result) {
  if (out_result == nullptr) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "out_result is null");
  }
  *out_result = nullptr;
  if (engine == nullptr) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "engine is null");
  }
  if (image == nullptr || !has_size(image->struct_size, sizeof(*image)) ||
      image->bytes == nullptr || image->length == 0) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, "image bytes are required");
  }

  RunOptions resolved = engine->defaults;
  if (options != nullptr) {
    if (!has_size(options->struct_size, kRequiredRunOptionsSize)) {
      return fail(AGUS_OCR_INVALID_ARGUMENT, "invalid run options");
    }
    resolved = run_options_from_c(*options);
  }

  try {
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
    if (engine->pipeline == AGUS_OCR_PIPELINE_GEMMA_MARKDOWN) {
      if (!engine->gemma_native) {
        return fail(AGUS_OCR_UNAVAILABLE,
                    "Gemma Markdown runtime is not initialized");
      }
      agus_ocr_run_options_t gemma_options =
          options == nullptr ? engine->gemma_defaults : run_options_to_c(resolved);
      try {
        *out_result = allocate_result(
            engine->gemma_native->Recognize(*image, gemma_options));
      } catch (const std::exception& error) {
        const std::string backend_error = error.what();
        const agus_ocr_backend_t failed_backend =
            engine->gemma_native->active_backend();
        if (!runtime_allows_onnx_backend_fallback(engine->requested_runtime) ||
            failed_backend == AGUS_OCR_BACKEND_CPU ||
            !is_ort_or_backend_failure_message(failed_backend, backend_error)) {
          if (failed_backend != AGUS_OCR_BACKEND_CPU &&
              is_ort_or_backend_failure_message(failed_backend, backend_error)) {
            mark_gemma_backend_unhealthy(failed_backend, backend_error);
          }
          throw;
        }

        mark_gemma_backend_unhealthy(failed_backend, backend_error);
        std::vector<std::string> retry_failures = {
            backend_display_name(failed_backend) + ": " + backend_error};
        for (agus_ocr_backend_t retry_backend :
             remaining_gemma_attempts_after_failure(engine->requested_runtime,
                                                    failed_backend)) {
          RuntimeOptions retry_runtime =
              runtime_for_backend(engine->requested_runtime, retry_backend);
          std::vector<std::string> fallback_warnings = {
              ppocr_fallback_warning(failed_backend, retry_backend,
                                     backend_error)};
          try {
            const std::string bundle_root =
                engine->external_model_root.empty() ? engine->model_root
                                                    : engine->external_model_root;
            const agus_ocr::GemmaMarkdownBundleCheck bundle =
                agus_ocr::ValidateGemmaMarkdownBundle(bundle_root);
            engine->gemma_native =
                std::make_unique<agus_ocr::GemmaMarkdownEngine>(
                    bundle, runtime_options_to_c(retry_runtime), retry_backend);
            engine->runtime = retry_runtime;
            *out_result = allocate_result(engine->gemma_native->Recognize(
                *image, gemma_options, fallback_warnings));
            break;
          } catch (const std::exception& retry_error) {
            const std::string retry_message = retry_error.what();
            retry_failures.push_back(backend_display_name(retry_backend) + ": " +
                                     retry_message);
            if (retry_backend != AGUS_OCR_BACKEND_CPU) {
              mark_gemma_backend_unhealthy(retry_backend, retry_message);
            }
          }
        }
        if (*out_result == nullptr) {
          std::ostringstream message;
          message << backend_display_name(failed_backend)
                  << " failed and all automatic Gemma Markdown retry backends "
                     "failed.";
          if (!retry_failures.empty()) {
            message << " Attempts: ";
            for (size_t i = 0; i < retry_failures.size(); ++i) {
              if (i > 0) {
                message << " | ";
              }
              message << retry_failures[i];
            }
          }
          return fail(AGUS_OCR_INTERNAL_ERROR, message.str());
        }
      }
      g_last_error.clear();
      return AGUS_OCR_OK;
    }
    if (engine->pipeline == AGUS_OCR_PIPELINE_PADDLEOCR_VL16) {
      if (!engine->vl_native) {
        return fail(AGUS_OCR_UNAVAILABLE,
                    "PaddleOCR-VL runtime is not initialized");
      }
      agus_ocr_run_options_t vl_options =
          options == nullptr ? engine->vl_defaults : run_options_to_c(resolved);
      *out_result = allocate_result(
          engine->vl_native->Recognize(*image, vl_options));
      g_last_error.clear();
      return AGUS_OCR_OK;
    }
    if (!engine->native) {
      return fail(AGUS_OCR_UNAVAILABLE, "native OCR runtime is not initialized");
    }
    try {
      *out_result = allocate_result(engine->native->recognize(*image, resolved));
    } catch (const std::exception& error) {
      const std::string backend_error = error.what();
      const agus_ocr_backend_t failed_backend =
          engine->native->active_backend();
      if (!should_retry_ppocr_recognition(*engine, backend_error)) {
        if (failed_backend != AGUS_OCR_BACKEND_CPU &&
            is_ort_or_backend_failure_message(failed_backend, backend_error)) {
          mark_backend_unhealthy(failed_backend, backend_error);
        }
        throw;
      }

      mark_backend_unhealthy(failed_backend, backend_error);
      std::vector<std::string> retry_failures = {
          backend_display_name(failed_backend) + ": " + backend_error};
      for (agus_ocr_backend_t retry_backend :
           remaining_ppocr_attempts_after_failure(engine->requested_runtime,
                                                  failed_backend)) {
        RuntimeOptions retry_runtime =
            runtime_for_backend(engine->requested_runtime, retry_backend);
        std::vector<std::string> fallback_warnings = {
            ppocr_fallback_warning(failed_backend, retry_backend,
                                   backend_error)};
        try {
          engine->native =
              std::make_unique<NativeEngine>(engine->model_root, retry_runtime);
          engine->runtime = retry_runtime;
          *out_result = allocate_result(
              engine->native->recognize(*image, resolved, fallback_warnings));
          break;
        } catch (const std::exception& retry_error) {
          const std::string retry_message = retry_error.what();
          retry_failures.push_back(backend_display_name(retry_backend) + ": " +
                                   retry_message);
          if (retry_backend != AGUS_OCR_BACKEND_CPU) {
            mark_backend_unhealthy(retry_backend, retry_message);
          }
        }
      }
      if (*out_result == nullptr) {
        std::ostringstream message;
        message << backend_display_name(failed_backend)
                << " failed and all automatic PP-OCRv6 retry backends failed.";
        if (!retry_failures.empty()) {
          message << " Attempts: ";
          for (size_t i = 0; i < retry_failures.size(); ++i) {
            if (i > 0) {
              message << " | ";
            }
            message << retry_failures[i];
          }
        }
        return fail(AGUS_OCR_INTERNAL_ERROR, message.str());
      }
    }
    g_last_error.clear();
    return AGUS_OCR_OK;
#else
    const std::string message =
        "native OCR core was built without ONNX Runtime/OpenCV support";
    *out_result = allocate_result(
        make_error_json(AGUS_OCR_UNAVAILABLE, message, "native-stub"));
    return AGUS_OCR_UNAVAILABLE;
#endif
  } catch (const std::invalid_argument& error) {
    return fail(AGUS_OCR_INVALID_ARGUMENT, error.what());
  } catch (const std::exception& error) {
    return fail(AGUS_OCR_INTERNAL_ERROR, error.what());
  }
}

void agus_ocr_free_result(agus_ocr_result_t* result) {
  if (result == nullptr) {
    return;
  }
  std::free(result->json);
  std::free(result);
}

void agus_ocr_destroy(agus_ocr_engine_t* engine) { delete engine; }

const char* agus_ocr_last_error(void) { return g_last_error.c_str(); }

const char* agus_ocr_engine_runtime_summary(agus_ocr_engine_t* engine) {
  static thread_local std::string summary;
  if (engine == nullptr) {
    summary.clear();
    return summary.c_str();
  }
#if defined(AGUS_OCR_ENABLE_NATIVE_PIPELINE)
  if (engine->pipeline == AGUS_OCR_PIPELINE_GEMMA_MARKDOWN &&
      engine->gemma_native) {
    summary = engine->gemma_native->runtime_summary();
    return summary.c_str();
  }
  if (engine->pipeline == AGUS_OCR_PIPELINE_PADDLEOCR_VL16 &&
      engine->vl_native) {
    summary = engine->vl_native->runtime_summary();
    return summary.c_str();
  }
  if (engine->native) {
    summary = engine->native->runtime_summary();
    return summary.c_str();
  }
#endif
  summary = make_runtime_summary(engine->runtime.backend,
                                 engine->runtime.cpu_threads);
  return summary.c_str();
}

}  // extern "C"
