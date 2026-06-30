#pragma once

#include <cstdint>
#include <string_view>
#include <vector>

namespace uocr {

struct ModelCatalogEntry {
  std::string_view model_id;
  std::string_view display_name;
  std::string_view model_file;
  std::string_view quantization;
  std::string_view quality;
  std::string_view hardware_tier;
  std::string_view notes;
  int bits = 0;
  std::uint64_t model_size_bytes = 0;
  bool recommended = false;
};

std::string_view default_model_id();
std::string_view provider_repo_id();
std::string_view provider_revision();
std::string_view provider_label();
std::string_view shared_mmproj_file();
std::uint64_t shared_mmproj_size_bytes();

const std::vector<ModelCatalogEntry>& unlimited_ocr_model_catalog();
const ModelCatalogEntry* find_model_catalog_entry(std::string_view model_id);

}  // namespace uocr
