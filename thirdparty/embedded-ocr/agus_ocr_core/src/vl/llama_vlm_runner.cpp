#include "vl/llama_vlm_runner.hpp"

#include <algorithm>
#include <array>
#include <cctype>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>

#include "opencv2/imgproc.hpp"
#include "vl/llama_vlm_support.hpp"

#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)
#include "ggml-backend.h"
#include "llama.h"
#include "mtmd-helper.h"
#include "mtmd.h"
#endif

namespace agus_ocr {
namespace {

#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)

void QuietLog(ggml_log_level, const char*, void*) {}

void EnsureLlamaInitialized() {
  static std::once_flag init_once;
  std::call_once(init_once, [] {
    llama_log_set(QuietLog, nullptr);
    mtmd_helper_log_set(QuietLog, nullptr);
    llama_backend_init();
    ggml_backend_load_all();
  });
}

std::string Lower(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(),
                 [](unsigned char c) {
                   return static_cast<char>(std::tolower(c));
                 });
  return value;
}

const char* GenerativeBackendLabel(agus_ocr_generative_backend_t backend) {
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

bool BackendCompiled(agus_ocr_generative_backend_t backend) {
  switch (backend) {
    case AGUS_OCR_GEN_BACKEND_CPU:
      return true;
    case AGUS_OCR_GEN_BACKEND_CUDA:
#if defined(AGUS_OCR_LLAMA_BACKEND_CUDA)
      return true;
#else
      return false;
#endif
    case AGUS_OCR_GEN_BACKEND_VULKAN:
#if defined(AGUS_OCR_LLAMA_BACKEND_VULKAN)
      return true;
#else
      return false;
#endif
    case AGUS_OCR_GEN_BACKEND_OPENCL:
#if defined(AGUS_OCR_LLAMA_BACKEND_OPENCL)
      return true;
#else
      return false;
#endif
    case AGUS_OCR_GEN_BACKEND_AUTO:
      return false;
  }
  return false;
}

bool DeviceMatchesBackend(ggml_backend_dev_t device,
                          agus_ocr_generative_backend_t backend) {
  if (device == nullptr) {
    return false;
  }
  const ggml_backend_reg_t reg = ggml_backend_dev_backend_reg(device);
  const char* raw_name = reg == nullptr ? nullptr : ggml_backend_reg_name(reg);
  const std::string name = raw_name == nullptr ? "" : Lower(raw_name);
  switch (backend) {
    case AGUS_OCR_GEN_BACKEND_CUDA:
      return name.find("cuda") != std::string::npos;
    case AGUS_OCR_GEN_BACKEND_VULKAN:
      return name.find("vulkan") != std::string::npos;
    case AGUS_OCR_GEN_BACKEND_OPENCL:
      return name.find("opencl") != std::string::npos;
    default:
      return false;
  }
}

ggml_backend_dev_t FindDeviceForBackend(agus_ocr_generative_backend_t backend) {
  if (backend == AGUS_OCR_GEN_BACKEND_CPU) {
    return nullptr;
  }
  EnsureLlamaInitialized();
  const size_t count = ggml_backend_dev_count();
  for (size_t i = 0; i < count; ++i) {
    ggml_backend_dev_t device = ggml_backend_dev_get(i);
    if (DeviceMatchesBackend(device, backend)) {
      return device;
    }
  }
  return nullptr;
}

LlamaVlmBackendInfo MakeBackendInfo(agus_ocr_generative_backend_t backend) {
  LlamaVlmBackendInfo info;
  info.backend = backend;
  if (backend == AGUS_OCR_GEN_BACKEND_CPU) {
    info.supported = true;
    info.device_name = "llama.cpp CPU";
    return info;
  }
  if (!BackendCompiled(backend)) {
    info.unavailable_reason =
        std::string("llama.cpp ") + GenerativeBackendLabel(backend) +
        " backend was not compiled into this build";
    return info;
  }
  ggml_backend_dev_t device = FindDeviceForBackend(backend);
  if (device == nullptr) {
    info.unavailable_reason =
        std::string("llama.cpp ") + GenerativeBackendLabel(backend) +
        " backend is compiled but no compatible device was reported";
    return info;
  }
  info.supported = true;
  const char* description = ggml_backend_dev_description(device);
  const char* name = ggml_backend_dev_name(device);
  info.device_name =
      description != nullptr && description[0] != '\0'
          ? description
          : (name == nullptr ? GenerativeBackendLabel(backend) : name);
  return info;
}

std::vector<LlamaVlmBackendInfo> BuildBackendInfos() {
  std::vector<LlamaVlmBackendInfo> infos = {
      MakeBackendInfo(AGUS_OCR_GEN_BACKEND_CPU),
      MakeBackendInfo(AGUS_OCR_GEN_BACKEND_CUDA),
      MakeBackendInfo(AGUS_OCR_GEN_BACKEND_VULKAN),
      MakeBackendInfo(AGUS_OCR_GEN_BACKEND_OPENCL),
  };
  agus_ocr_generative_backend_t default_backend = AGUS_OCR_GEN_BACKEND_CPU;
  for (const auto& info : infos) {
    if (info.supported && info.backend != AGUS_OCR_GEN_BACKEND_CPU) {
      default_backend = info.backend;
      break;
    }
  }
  for (auto& info : infos) {
    info.enabled_by_default = info.backend == default_backend;
  }
  return infos;
}

const LlamaVlmBackendInfo& BackendInfoOrThrow(
    agus_ocr_generative_backend_t backend) {
  static const std::vector<LlamaVlmBackendInfo> infos = BuildBackendInfos();
  for (const auto& info : infos) {
    if (info.backend == backend) {
      if (!info.supported) {
        throw std::runtime_error(info.unavailable_reason);
      }
      return info;
    }
  }
  throw std::runtime_error("unsupported PaddleOCR-VL generative backend");
}

agus_ocr_generative_backend_t ResolveBackend(
    const agus_ocr_runtime_options_t& runtime) {
  if (runtime.force_cpu_only != 0 ||
      runtime.generative_backend == AGUS_OCR_GEN_BACKEND_CPU) {
    return AGUS_OCR_GEN_BACKEND_CPU;
  }
  if (runtime.generative_backend != AGUS_OCR_GEN_BACKEND_AUTO) {
    return runtime.generative_backend;
  }
  static const std::vector<LlamaVlmBackendInfo> infos = BuildBackendInfos();
  for (const auto& info : infos) {
    if (info.supported && info.enabled_by_default) {
      return info.backend;
    }
  }
  return AGUS_OCR_GEN_BACKEND_CPU;
}

struct LlamaModelDeleter {
  void operator()(llama_model* model) const { llama_model_free(model); }
};
struct LlamaContextDeleter {
  void operator()(llama_context* ctx) const { llama_free(ctx); }
};
struct MtmdContextDeleter {
  void operator()(mtmd_context* ctx) const { mtmd_free(ctx); }
};
struct MtmdBitmapDeleter {
  void operator()(mtmd_bitmap* bitmap) const { mtmd_bitmap_free(bitmap); }
};
struct MtmdChunksDeleter {
  void operator()(mtmd_input_chunks* chunks) const {
    mtmd_input_chunks_free(chunks);
  }
};
struct LlamaSamplerDeleter {
  void operator()(llama_sampler* sampler) const { llama_sampler_free(sampler); }
};

using LlamaModelPtr = std::unique_ptr<llama_model, LlamaModelDeleter>;
using LlamaContextPtr = std::unique_ptr<llama_context, LlamaContextDeleter>;
using MtmdContextPtr = std::unique_ptr<mtmd_context, MtmdContextDeleter>;
using MtmdBitmapPtr = std::unique_ptr<mtmd_bitmap, MtmdBitmapDeleter>;
using MtmdChunksPtr = std::unique_ptr<mtmd_input_chunks, MtmdChunksDeleter>;
using LlamaSamplerPtr = std::unique_ptr<llama_sampler, LlamaSamplerDeleter>;

std::string TokenToPiece(const llama_vocab* vocab, llama_token token) {
  char stack_buffer[256];
  int size = llama_token_to_piece(vocab, token, stack_buffer,
                                  sizeof(stack_buffer), 0, false);
  if (size >= 0) {
    return std::string(stack_buffer, static_cast<size_t>(size));
  }
  std::vector<char> buffer(static_cast<size_t>(-size));
  size = llama_token_to_piece(vocab, token, buffer.data(),
                              static_cast<int32_t>(buffer.size()), 0, false);
  if (size < 0) {
    return "";
  }
  return std::string(buffer.data(), static_cast<size_t>(size));
}

LlamaSamplerPtr MakeSampler(float temperature) {
  auto params = llama_sampler_chain_default_params();
  params.no_perf = true;
  LlamaSamplerPtr sampler(llama_sampler_chain_init(params));
  if (!sampler) {
    throw std::runtime_error("failed to create llama sampler");
  }
  if (temperature <= 0.0f) {
    llama_sampler_chain_add(sampler.get(), llama_sampler_init_greedy());
  } else {
    llama_sampler_chain_add(sampler.get(), llama_sampler_init_top_k(40));
    llama_sampler_chain_add(sampler.get(), llama_sampler_init_top_p(0.95f, 1));
    llama_sampler_chain_add(sampler.get(),
                            llama_sampler_init_temp(temperature));
    llama_sampler_chain_add(sampler.get(),
                            llama_sampler_init_dist(LLAMA_DEFAULT_SEED));
  }
  return sampler;
}

std::string BuildPrompt(const std::string& label, bool markdown) {
  return std::string("<|begin_of_sentence|>User: ") + mtmd_default_marker() +
         PromptForLayoutLabel(label, markdown) + "\nAssistant:\n";
}

#endif

}  // namespace

#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)

class LlamaVlmRunner::Impl {
 public:
  Impl(const std::string& model_path, const std::string& mmproj_path,
       const agus_ocr_runtime_options_t& runtime)
      : threads_(ResolveLlamaThreads(runtime)),
        active_backend_(ResolveBackend(runtime)) {
    EnsureLlamaInitialized();
    const LlamaVlmBackendInfo& backend_info =
        BackendInfoOrThrow(active_backend_);
    if (active_backend_ != AGUS_OCR_GEN_BACKEND_CPU) {
      selected_device_ = FindDeviceForBackend(active_backend_);
      if (selected_device_ == nullptr) {
        throw std::runtime_error(backend_info.unavailable_reason);
      }
      gpu_layers_ =
          runtime.generative_gpu_layers > 0 ? runtime.generative_gpu_layers : -1;
    }

    llama_model_params model_params = llama_model_default_params();
    std::array<ggml_backend_dev_t, 2> devices = {selected_device_, nullptr};
    if (selected_device_ != nullptr) {
      model_params.devices = devices.data();
    }
    model_params.n_gpu_layers = gpu_layers_;
    model_params.use_mmap = true;
    model_.reset(llama_model_load_from_file(model_path.c_str(), model_params));
    if (!model_) {
      throw std::runtime_error("failed to load PaddleOCR-VL GGUF model");
    }

    llama_context_params context_params = llama_context_default_params();
    context_params.n_ctx = 8192;
    context_params.n_batch = 512;
    context_params.n_ubatch = 512;
    context_params.n_threads = threads_;
    context_params.n_threads_batch = threads_;
    context_params.no_perf = true;
    context_.reset(llama_init_from_model(model_.get(), context_params));
    if (!context_) {
      throw std::runtime_error("failed to create PaddleOCR-VL llama context");
    }

    mtmd_context_params mtmd_params = mtmd_context_params_default();
    mtmd_params.use_gpu = active_backend_ != AGUS_OCR_GEN_BACKEND_CPU;
    mtmd_params.print_timings = false;
    mtmd_params.n_threads = threads_;
    mtmd_params.warmup = false;
    mtmd_.reset(mtmd_init_from_file(mmproj_path.c_str(), model_.get(),
                                    mtmd_params));
    if (!mtmd_ || !mtmd_support_vision(mtmd_.get())) {
      throw std::runtime_error("failed to load PaddleOCR-VL mmproj model");
    }
  }

  std::string Recognize(const cv::Mat& bgr,
                        const std::string& label,
                        const VlGenerationOptions& options) {
    std::lock_guard<std::mutex> lock(mutex_);
    llama_memory_clear(llama_get_memory(context_.get()), true);

    cv::Mat prepared = ResizeToPixelBudget(bgr, options);
    cv::Mat rgb;
    cv::cvtColor(prepared, rgb, cv::COLOR_BGR2RGB);
    MtmdBitmapPtr bitmap(mtmd_bitmap_init(
        static_cast<uint32_t>(rgb.cols), static_cast<uint32_t>(rgb.rows),
        rgb.data));
    if (!bitmap) {
      throw std::runtime_error("failed to create PaddleOCR-VL image bitmap");
    }

    const std::string prompt = BuildPrompt(label, options.generate_markdown);
    mtmd_input_text text{prompt.c_str(), false, true};
    MtmdChunksPtr chunks(mtmd_input_chunks_init());
    const mtmd_bitmap* bitmap_ptr = bitmap.get();
    const int32_t tokenized =
        mtmd_tokenize(mtmd_.get(), chunks.get(), &text, &bitmap_ptr, 1);
    if (tokenized != 0) {
      throw std::runtime_error("failed to tokenize PaddleOCR-VL prompt");
    }

    llama_pos n_past = 0;
    const size_t chunk_count = mtmd_input_chunks_size(chunks.get());
    for (size_t i = 0; i < chunk_count; ++i) {
      llama_pos new_n_past = n_past;
      const int32_t result = mtmd_helper_eval_chunk_single(
          mtmd_.get(), context_.get(), mtmd_input_chunks_get(chunks.get(), i),
          n_past, 0, 512, i + 1 == chunk_count, &new_n_past);
      if (result != 0) {
        throw std::runtime_error("failed to evaluate PaddleOCR-VL prompt");
      }
      n_past = new_n_past;
    }

    LlamaSamplerPtr sampler = MakeSampler(options.temperature);
    llama_batch batch = llama_batch_init(1, 0, 1);
    std::string generated;
    const llama_vocab* vocab = llama_model_get_vocab(model_.get());
    const int max_tokens = std::max(1, options.max_new_tokens);
    for (int i = 0; i < max_tokens; ++i) {
      const llama_token token =
          llama_sampler_sample(sampler.get(), context_.get(), -1);
      if (llama_vocab_is_eog(vocab, token)) {
        break;
      }
      generated += TokenToPiece(vocab, token);

      batch.n_tokens = 1;
      batch.token[0] = token;
      batch.pos[0] = n_past++;
      batch.n_seq_id[0] = 1;
      batch.seq_id[0][0] = 0;
      batch.logits[0] = 1;
      if (llama_decode(context_.get(), batch) != 0) {
        llama_batch_free(batch);
        throw std::runtime_error("failed to decode PaddleOCR-VL token");
      }
    }
    llama_batch_free(batch);
    return TrimGeneratedText(generated);
  }

  std::string runtime_summary() const {
    return std::string("llama.cpp mtmd ") +
           GenerativeBackendLabel(active_backend_) +
           " gpuLayers=" + std::to_string(gpu_layers_) +
           " threads=" + std::to_string(threads_);
  }

 private:
  int threads_ = 1;
  agus_ocr_generative_backend_t active_backend_ = AGUS_OCR_GEN_BACKEND_CPU;
  int gpu_layers_ = 0;
  ggml_backend_dev_t selected_device_ = nullptr;
  std::mutex mutex_;
  LlamaModelPtr model_;
  LlamaContextPtr context_;
  MtmdContextPtr mtmd_;
};

bool LlamaVlmRuntimeAvailable() { return true; }

std::string LlamaVlmUnavailableReason() { return ""; }

std::vector<LlamaVlmBackendInfo> LlamaVlmBackendInfos() {
  return BuildBackendInfos();
}

agus_ocr_generative_backend_t LlamaVlmDefaultBackend() {
  for (const auto& info : BuildBackendInfos()) {
    if (info.supported && info.enabled_by_default) {
      return info.backend;
    }
  }
  return AGUS_OCR_GEN_BACKEND_CPU;
}

#else

class LlamaVlmRunner::Impl {};

bool LlamaVlmRuntimeAvailable() { return false; }

std::string LlamaVlmUnavailableReason() {
  return "this build was compiled without the pinned llama.cpp dependency; run "
         "`dart run tool/fetch_llama_cpp.dart` before building native targets";
}

std::vector<LlamaVlmBackendInfo> LlamaVlmBackendInfos() {
  LlamaVlmBackendInfo cpu;
  cpu.backend = AGUS_OCR_GEN_BACKEND_CPU;
  cpu.unavailable_reason = LlamaVlmUnavailableReason();
  return {cpu};
}

agus_ocr_generative_backend_t LlamaVlmDefaultBackend() {
  return AGUS_OCR_GEN_BACKEND_CPU;
}

#endif

LlamaVlmRunner::LlamaVlmRunner(const std::string& model_path,
                               const std::string& mmproj_path,
                               const agus_ocr_runtime_options_t& runtime)
#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)
    : impl_(std::make_unique<Impl>(model_path, mmproj_path, runtime))
#else
    : impl_(std::make_unique<Impl>())
#endif
{
#if !defined(AGUS_OCR_ENABLE_LLAMA_CPP)
  (void)model_path;
  (void)mmproj_path;
  (void)runtime;
  throw std::runtime_error(LlamaVlmUnavailableReason());
#endif
}

LlamaVlmRunner::~LlamaVlmRunner() = default;

std::string LlamaVlmRunner::Recognize(const cv::Mat& bgr,
                                      const std::string& label,
                                      const VlGenerationOptions& options) {
#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)
  return impl_->Recognize(bgr, label, options);
#else
  (void)bgr;
  (void)label;
  (void)options;
  throw std::runtime_error(LlamaVlmUnavailableReason());
#endif
}

std::string LlamaVlmRunner::runtime_summary() const {
#if defined(AGUS_OCR_ENABLE_LLAMA_CPP)
  return impl_->runtime_summary();
#else
  return "llama.cpp unavailable";
#endif
}

}  // namespace agus_ocr
