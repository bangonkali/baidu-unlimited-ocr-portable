#include "uocr/core/runtime_catalog.hpp"

#include <cstdlib>
#include <string>
#include <vector>

namespace uocr {
namespace {

struct RuntimeSpec {
  std::string_view platform;
  std::string_view label;
  std::string_view accelerator;
  std::string_view backend;
  std::string_view library_name;
  int priority;
  int n_gpu_layers;
};

bool file_exists(const std::filesystem::path& path) {
  std::error_code error;
  return std::filesystem::exists(path, error) && !std::filesystem::is_directory(path, error);
}

bool env_flag(std::string_view name) {
  const std::string key(name);
  const char* value = std::getenv(key.c_str());
  return value != nullptr && value[0] != '\0' && std::string_view(value) != "0";
}

bool executable_exists(const std::filesystem::path& path) {
  return file_exists(path);
}

bool path_command_exists(std::string_view command) {
  const char* raw_path = std::getenv("PATH");
  if (raw_path == nullptr) {
    return false;
  }
#ifdef _WIN32
  constexpr char separator = ';';
  const std::string suffix = ".exe";
#else
  constexpr char separator = ':';
  const std::string suffix;
#endif
  std::string paths(raw_path);
  std::size_t start = 0;
  while (start <= paths.size()) {
    const auto end = paths.find(separator, start);
    const auto part = paths.substr(start, end == std::string::npos ? std::string::npos : end - start);
    if (!part.empty()) {
      auto candidate = std::filesystem::path(part) / std::string(command);
      if (executable_exists(candidate) || (!suffix.empty() && executable_exists(candidate.string() + suffix))) {
        return true;
      }
    }
    if (end == std::string::npos) {
      break;
    }
    start = end + 1;
  }
  return false;
}

bool windows_file_exists(std::string_view raw) {
#ifdef _WIN32
  return file_exists(std::filesystem::path(std::string(raw)));
#else
  (void)raw;
  return false;
#endif
}

bool cuda_available() {
  return env_flag("UOCR_FORCE_CUDA") || path_command_exists("nvidia-smi") ||
         windows_file_exists("C:/Windows/System32/nvidia-smi.exe");
}

bool rocm_available() {
  return env_flag("UOCR_FORCE_ROCM") || path_command_exists("rocminfo") || path_command_exists("hipinfo") ||
         windows_file_exists("C:/Program Files/AMD/ROCm/bin/rocminfo.exe") ||
         windows_file_exists("C:/Program Files/AMD/ROCm/bin/hipinfo.exe");
}

std::vector<RuntimeSpec> runtime_specs() {
#ifdef _WIN32
  return {
      {"windows-x86_64-cuda13", "Windows x64 CUDA 13", "cuda", "cuda", "uocr-ffi.dll", 300, -2},
      {"windows-x86_64-rocm6", "Windows x64 AMD ROCm/HIP", "rocm", "rocm", "uocr-ffi.dll", 200, -2},
      {"windows-x86_64-cpu", "Windows x64 CPU", "cpu", "cpu", "uocr-ffi.dll", 100, 0},
  };
#elif defined(__APPLE__)
  return {
      {"macos-arm64-metal", "macOS Apple Silicon Metal", "metal", "metal", "libuocr-ffi.dylib", 300, -2},
      {"macos-arm64-cpu", "macOS Apple Silicon CPU", "cpu", "cpu", "libuocr-ffi.dylib", 100, 0},
  };
#else
  return {
      {"linux-x86_64-cuda13", "Linux x64 CUDA 13", "cuda", "cuda", "libuocr-ffi.so", 300, -2},
      {"linux-x86_64-rocm6", "Linux x64 AMD ROCm/HIP", "rocm", "rocm", "libuocr-ffi.so", 200, -2},
      {"linux-x86_64-cpu", "Linux x64 CPU", "cpu", "cpu", "libuocr-ffi.so", 100, 0},
  };
#endif
}

bool hardware_supported_for(const RuntimeHardwareProbe& probe, std::string_view accelerator) {
  if (accelerator == "cuda") {
    return probe.cuda;
  }
  if (accelerator == "rocm") {
    return probe.rocm;
  }
  if (accelerator == "metal") {
    return probe.metal;
  }
  return probe.cpu;
}

std::string detail_for(const RuntimeHardwareProbe& probe, std::string_view accelerator) {
  if (accelerator == "cuda") {
    return probe.cuda_detail;
  }
  if (accelerator == "rocm") {
    return probe.rocm_detail;
  }
  if (accelerator == "metal") {
    return probe.metal_detail;
  }
  return probe.cpu_detail;
}

}  // namespace

RuntimeHardwareProbe detect_runtime_hardware() {
  RuntimeHardwareProbe probe;
  probe.cuda = cuda_available();
  probe.rocm = rocm_available();
#ifdef __APPLE__
  probe.metal = true;
  probe.metal_detail = "Apple Metal runtime is available on Apple Silicon";
#endif
  probe.cuda_detail = probe.cuda ? "NVIDIA CUDA probe found" : "nvidia-smi was not found";
  probe.rocm_detail = probe.rocm ? "AMD ROCm/HIP probe found" : "rocminfo/hipinfo was not found";
  return probe;
}

std::vector<RuntimeVariant> runtime_variants_for(const std::filesystem::path& app_root,
                                                 const RuntimeHardwareProbe& probe) {
  std::vector<RuntimeVariant> variants;
  const auto runtime_root = app_root / "thirdparty" / "uocr-runtime";
  for (const auto& spec : runtime_specs()) {
    const auto ffi = runtime_root / std::string(spec.platform) / "bin" / std::string(spec.library_name);
    RuntimeVariant variant;
    variant.runtime_id = std::string(spec.platform);
    variant.platform = std::string(spec.platform);
    variant.label = std::string(spec.label);
    variant.accelerator = std::string(spec.accelerator);
    variant.backend = std::string(spec.backend);
    variant.library_name = std::string(spec.library_name);
    variant.ffi_library = ffi;
    variant.priority = spec.priority;
    variant.n_gpu_layers = spec.n_gpu_layers;
    variant.installed = file_exists(ffi);
    variant.hardware_supported = hardware_supported_for(probe, spec.accelerator);
    variant.selectable = variant.installed && variant.hardware_supported;
    variant.support_detail = detail_for(probe, spec.accelerator);
    variants.push_back(std::move(variant));
  }
  return variants;
}

std::string choose_runtime_id(const std::vector<RuntimeVariant>& variants, std::string_view preferred) {
  const auto* saved = find_runtime_variant(variants, preferred);
  if (saved != nullptr && saved->selectable) {
    return saved->runtime_id;
  }
  const RuntimeVariant* best = nullptr;
  for (const auto& variant : variants) {
    if (!variant.selectable) {
      continue;
    }
    if (best == nullptr || variant.priority > best->priority) {
      best = &variant;
    }
  }
  return best != nullptr ? best->runtime_id : (variants.empty() ? std::string() : variants.front().runtime_id);
}

const RuntimeVariant* find_runtime_variant(const std::vector<RuntimeVariant>& variants,
                                           std::string_view runtime_id) {
  for (const auto& variant : variants) {
    if (variant.runtime_id == runtime_id) {
      return &variant;
    }
  }
  return nullptr;
}

}  // namespace uocr
