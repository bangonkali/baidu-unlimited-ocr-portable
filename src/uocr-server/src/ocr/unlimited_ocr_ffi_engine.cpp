#include "uocr/ocr/unlimited_ocr_ffi_engine.hpp"

#include <cstdint>
#include <sstream>
#include <stdexcept>
#include <string>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <dlfcn.h>
#endif

namespace uocr {
namespace {

constexpr std::uint32_t kExpectedAbiVersion = 1;
constexpr std::uint32_t kEventToken = 1;
constexpr std::int32_t kStatusOk = 0;

struct UocrFfiEvent {
  std::uint32_t struct_size;
  std::uint32_t type;
  const void* text_utf8;
  std::uint64_t text_len;
  const void* json_utf8;
  std::uint64_t json_len;
  std::int32_t code;
  std::uint32_t reserved_u32;
  std::uint64_t index;
  void* reserved_ptr0;
  void* reserved_ptr1;
  void* reserved_ptr2;
  void* reserved_ptr3;
};

using UocrFfiEventCallback = std::int32_t (*)(const UocrFfiEvent*, void*);

struct UocrFfiParams {
  std::uint32_t struct_size;
  std::uint32_t flags;
  const char* model_path;
  const char* mmproj_path;
  const char* chat_template;
  std::int32_t ctx_size;
  std::int32_t n_batch;
  std::int32_t n_gpu_layers;
  std::int32_t log_verbosity;
  std::int32_t force_prompt_eos;
  std::int32_t no_image_end;
  std::int32_t gundam_mode;
  std::int32_t no_repeat_ngram;
  std::int32_t ngram_size;
  std::int32_t ngram_window;
  const char* ngram_whitelist;
  std::int32_t prefill_aware_swa;
  std::int32_t legacy_kv_prune;
  std::int32_t decode_window;
  std::int32_t min_new_tokens;
  void* reserved_ptr0;
  void* reserved_ptr1;
  void* reserved_ptr2;
  void* reserved_ptr3;
};

struct UocrFfiRequest {
  std::uint32_t struct_size;
  std::uint32_t flags;
  const char* image_path;
  const char* prompt;
  std::int32_t max_tokens;
  std::int32_t reserved_i32;
  UocrFfiEventCallback event_callback;
  void* user_data;
  void* reserved_ptr0;
  void* reserved_ptr1;
  void* reserved_ptr2;
  void* reserved_ptr3;
};

struct CallbackState {
  OcrResult* result;
  const std::function<void(const OcrEvent&)>* sink;
};

std::string join_whitelist(const std::vector<int>& values) {
  std::ostringstream out;
  for (std::size_t index = 0; index < values.size(); ++index) {
    if (index > 0) {
      out << ',';
    }
    out << values[index];
  }
  return out.str();
}

std::string format_prompt(const std::string& prompt, const std::string& media_placement) {
  const std::string marker = "<image>";
  if (prompt.find(marker) != std::string::npos) {
    return prompt;
  }
  if (media_placement == "prefix-tight") {
    return marker + prompt;
  }
  if (media_placement == "suffix-newline") {
    return prompt + "\n" + marker;
  }
  return marker + "\n" + prompt;
}

std::int32_t on_ffi_event(const UocrFfiEvent* event, void* user_data) {
  if (event == nullptr || user_data == nullptr || event->type != kEventToken || event->text_utf8 == nullptr) {
    return 0;
  }
  auto* state = static_cast<CallbackState*>(user_data);
  std::string text(static_cast<const char*>(event->text_utf8), static_cast<std::size_t>(event->text_len));
  state->result->text += text;
  if (state->sink != nullptr) {
    (*state->sink)(OcrEvent{.kind = OcrEvent::Kind::Token, .text = text, .index = event->index});
  }
  return 0;
}

std::string dynamic_library_error() {
#ifdef _WIN32
  const auto code = GetLastError();
  return code == 0 ? std::string{} : " (GetLastError=" + std::to_string(code) + ")";
#else
  const char* message = dlerror();
  return message == nullptr ? std::string{} : ": " + std::string(message);
#endif
}

}  // namespace

struct UnlimitedOcrFfiEngine::Impl {
  using AbiVersionFn = std::uint32_t (*)();
  using CreateFn = void* (*)(const UocrFfiParams*);
  using DestroyFn = void (*)(void*);
  using RunImageFn = std::int32_t (*)(void*, const UocrFfiRequest*);
  using LastErrorFn = const char* (*)(void*);
  using RunCountFn = std::uint64_t (*)(void*);

  UnlimitedOcrRuntimePaths paths;
  OcrProfileRecord profile;
  void* library = nullptr;
  void* session = nullptr;
  CreateFn create = nullptr;
  DestroyFn destroy = nullptr;
  RunImageFn run_image = nullptr;
  LastErrorFn last_error = nullptr;
  RunCountFn run_count = nullptr;

  ~Impl() {
    if (session != nullptr && destroy != nullptr) {
      destroy(session);
    }
#ifdef _WIN32
    if (library != nullptr) {
      FreeLibrary(static_cast<HMODULE>(library));
    }
#else
    if (library != nullptr) {
      dlclose(library);
    }
#endif
  }

  template <typename T>
  T symbol(const char* name) {
#ifdef _WIN32
    auto* raw = reinterpret_cast<void*>(GetProcAddress(static_cast<HMODULE>(library), name));
#else
    auto* raw = dlsym(library, name);
#endif
    if (raw == nullptr) {
      throw std::runtime_error(std::string("uocr-ffi is missing symbol: ") + name);
    }
    return reinterpret_cast<T>(raw);
  }

  void load_library() {
    if (library != nullptr) {
      return;
    }
#ifdef _WIN32
    library = LoadLibraryExA(paths.ffi_library.string().c_str(), nullptr, LOAD_WITH_ALTERED_SEARCH_PATH);
#else
    library = dlopen(paths.ffi_library.string().c_str(), RTLD_NOW | RTLD_LOCAL);
#endif
    if (library == nullptr) {
      throw std::runtime_error("failed to load uocr-ffi library: " + paths.ffi_library.string() +
                               dynamic_library_error());
    }

    auto abi_version = symbol<AbiVersionFn>("uocr_ffi_abi_version");
    create = symbol<CreateFn>("uocr_ffi_create");
    destroy = symbol<DestroyFn>("uocr_ffi_destroy");
    run_image = symbol<RunImageFn>("uocr_ffi_run_image");
    last_error = symbol<LastErrorFn>("uocr_ffi_last_error");
    run_count = symbol<RunCountFn>("uocr_ffi_run_count");
    if (abi_version() != kExpectedAbiVersion) {
      throw std::runtime_error("unsupported uocr-ffi ABI version");
    }
  }

  void ensure_session() {
    load_library();
    if (session != nullptr) {
      return;
    }

    const std::string model = paths.model.string();
    const std::string mmproj = paths.mmproj.string();
    const std::string whitelist = join_whitelist(profile.ngram_whitelist);
    UocrFfiParams params{
        .struct_size = sizeof(UocrFfiParams),
        .flags = 0,
        .model_path = model.c_str(),
        .mmproj_path = mmproj.c_str(),
        .chat_template = "deepseek-ocr",
        .ctx_size = profile.ctx_size,
        .n_batch = 2048,
        .n_gpu_layers = paths.n_gpu_layers,
        .log_verbosity = 2,
        .force_prompt_eos = profile.force_prompt_eos ? 1 : 0,
        .no_image_end = profile.no_image_end ? 1 : 0,
        .gundam_mode = profile.deepseek_ocr_mode == "gundam" ? 1 : 0,
        .no_repeat_ngram = profile.no_repeat_ngram ? 1 : 0,
        .ngram_size = profile.ngram_size,
        .ngram_window = profile.ngram_window,
        .ngram_whitelist = whitelist.c_str(),
        .prefill_aware_swa = profile.prefill_aware_swa ? 1 : 0,
        .legacy_kv_prune = 0,
        .decode_window = profile.decode_window,
        .min_new_tokens = 0,
    };
    session = create(&params);
    if (session == nullptr) {
      const char* message = last_error != nullptr ? last_error(nullptr) : nullptr;
      throw std::runtime_error(message != nullptr ? message : "failed to create uocr-ffi session");
    }
  }
};

UnlimitedOcrFfiEngine::UnlimitedOcrFfiEngine(UnlimitedOcrRuntimePaths paths, OcrProfileRecord profile)
    : impl_(std::make_unique<Impl>(Impl{.paths = std::move(paths), .profile = std::move(profile)})) {}

UnlimitedOcrFfiEngine::~UnlimitedOcrFfiEngine() = default;

std::string UnlimitedOcrFfiEngine::id() const {
  return "unlimited-ocr-ffi";
}

OcrResult UnlimitedOcrFfiEngine::recognize_image(const OcrRequest& request,
                                                 const std::function<void(const OcrEvent&)>& event_sink) {
  OcrResult result;
  if (!std::filesystem::exists(request.image_path)) {
    result.error = "image path does not exist";
    return result;
  }

  try {
    impl_->ensure_session();
    const std::string image_path = request.image_path.string();
    const std::string prompt = format_prompt(request.prompt, impl_->profile.media_placement);
    CallbackState state{&result, &event_sink};
    UocrFfiRequest ffi_request{
        .struct_size = sizeof(UocrFfiRequest),
        .flags = 0,
        .image_path = image_path.c_str(),
        .prompt = prompt.c_str(),
        .max_tokens = request.max_tokens,
        .reserved_i32 = 0,
        .event_callback = on_ffi_event,
        .user_data = &state,
    };
    result.status_code = impl_->run_image(impl_->session, &ffi_request);
    result.ok = result.status_code == kStatusOk;
    result.run_count = impl_->run_count != nullptr ? impl_->run_count(impl_->session) : 0;
    if (!result.ok) {
      const char* message = impl_->last_error != nullptr ? impl_->last_error(impl_->session) : nullptr;
      result.error = message != nullptr ? message : "uocr-ffi returned an error";
      event_sink(OcrEvent{.kind = OcrEvent::Kind::Error, .message = result.error});
    } else {
      event_sink(OcrEvent{.kind = OcrEvent::Kind::Done, .text = result.text});
    }
  } catch (const std::exception& error) {
    result.error = error.what();
    event_sink(OcrEvent{.kind = OcrEvent::Kind::Error, .message = result.error});
  }
  return result;
}

}  // namespace uocr
