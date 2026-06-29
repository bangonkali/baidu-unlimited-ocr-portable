#include "uocr/fs/file_scanner.hpp"

#include <algorithm>
#include <array>
#include <cctype>
#include <stdexcept>

namespace uocr {
namespace {

std::string lowercase(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return value;
}

std::chrono::system_clock::time_point to_system_clock(std::filesystem::file_time_type value) {
  const auto now_file = std::filesystem::file_time_type::clock::now();
  const auto now_system = std::chrono::system_clock::now();
  return now_system + std::chrono::duration_cast<std::chrono::system_clock::duration>(value - now_file);
}

}  // namespace

bool is_supported_document_extension(const std::filesystem::path& path) {
  static constexpr std::array<const char*, 8> supported = {
      ".pdf", ".bmp", ".jpeg", ".jpg", ".png", ".tif", ".tiff", ".webp",
  };
  const std::string extension = lowercase(path.extension().string());
  return std::find(supported.begin(), supported.end(), extension) != supported.end();
}

std::filesystem::path validate_trusted_root(const std::filesystem::path& root) {
  std::error_code error;
  const auto status = std::filesystem::symlink_status(root, error);
  if (error || !std::filesystem::exists(status)) {
    throw std::invalid_argument("folder does not exist");
  }
  if (std::filesystem::is_symlink(status)) {
    throw std::invalid_argument("folder symlinks are not accepted");
  }
  if (!std::filesystem::is_directory(status)) {
    throw std::invalid_argument("path is not a folder");
  }

  auto canonical = std::filesystem::canonical(root, error);
  if (error) {
    throw std::invalid_argument("folder cannot be resolved");
  }
  return canonical;
}

std::vector<DiscoveredFile> discover_supported_files(const std::filesystem::path& root) {
  const auto safe_root = validate_trusted_root(root);
  std::vector<DiscoveredFile> files;
  std::error_code error;
  std::filesystem::recursive_directory_iterator it(
      safe_root, std::filesystem::directory_options::skip_permission_denied, error);

  for (std::filesystem::recursive_directory_iterator end; !error && it != end; it.increment(error)) {
    const auto status = it->symlink_status(error);
    if (error) {
      error.clear();
      continue;
    }
    if (std::filesystem::is_symlink(status)) {
      it.disable_recursion_pending();
      continue;
    }
    if (!std::filesystem::is_regular_file(status) || !is_supported_document_extension(it->path())) {
      continue;
    }

    DiscoveredFile file;
    file.absolute_path = std::filesystem::canonical(it->path(), error);
    if (error) {
      error.clear();
      continue;
    }
    file.relative_path = std::filesystem::relative(file.absolute_path, safe_root, error);
    file.size_bytes = std::filesystem::file_size(file.absolute_path, error);
    file.modified_at = to_system_clock(std::filesystem::last_write_time(file.absolute_path, error));
    files.push_back(std::move(file));
  }

  std::sort(files.begin(), files.end(), [](const DiscoveredFile& left, const DiscoveredFile& right) {
    return left.relative_path.generic_string() < right.relative_path.generic_string();
  });
  return files;
}

}  // namespace uocr
