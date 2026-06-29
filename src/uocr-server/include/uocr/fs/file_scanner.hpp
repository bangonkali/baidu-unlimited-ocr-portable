#pragma once

#include <chrono>
#include <filesystem>
#include <string>
#include <vector>

namespace uocr {

struct DiscoveredFile {
  std::filesystem::path absolute_path;
  std::filesystem::path relative_path;
  std::uintmax_t size_bytes = 0;
  std::chrono::system_clock::time_point modified_at;
};

bool is_supported_document_extension(const std::filesystem::path& path);
std::filesystem::path validate_trusted_root(const std::filesystem::path& root);
std::vector<DiscoveredFile> discover_supported_files(const std::filesystem::path& root);

}  // namespace uocr

