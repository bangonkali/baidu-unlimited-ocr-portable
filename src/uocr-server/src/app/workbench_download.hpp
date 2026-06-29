#pragma once

#include <cstdint>
#include <filesystem>
#include <functional>
#include <string>

namespace uocr::server {

struct DownloadProgress {
  std::uint64_t downloaded_bytes = 0;
  std::uint64_t total_bytes = 0;
  std::string current_file;
};

using DownloadProgressCallback = std::function<void(const DownloadProgress&)>;

void download_to_file(const std::string& url,
                      const std::filesystem::path& destination,
                      const DownloadProgressCallback& progress);

}  // namespace uocr::server
