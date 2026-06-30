#pragma once

#include <filesystem>
#include <string>
#include <string_view>
#include <vector>

namespace uocr {

struct RuntimeHardwareProbe {
  bool cuda = false;
  bool rocm = false;
  bool metal = false;
  bool cpu = true;
  std::string cuda_detail;
  std::string rocm_detail;
  std::string metal_detail;
  std::string cpu_detail = "CPU inference is available";
};

struct RuntimeVariant {
  std::string runtime_id;
  std::string label;
  std::string platform;
  std::string accelerator;
  std::string backend;
  std::filesystem::path ffi_library;
  std::string library_name;
  std::string support_detail;
  bool hardware_supported = false;
  bool installed = false;
  bool selectable = false;
  bool recommended = false;
  int priority = 0;
  int n_gpu_layers = -2;
};

RuntimeHardwareProbe detect_runtime_hardware();
std::vector<RuntimeVariant> runtime_variants_for(const std::filesystem::path& app_root,
                                                 const RuntimeHardwareProbe& probe);
std::string choose_runtime_id(const std::vector<RuntimeVariant>& variants, std::string_view preferred);
const RuntimeVariant* find_runtime_variant(const std::vector<RuntimeVariant>& variants,
                                           std::string_view runtime_id);

}  // namespace uocr
