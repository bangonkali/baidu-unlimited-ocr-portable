#pragma once

#include <atomic>
#include <cstdint>
#include <filesystem>
#include <functional>
#include <stdexcept>
#include <string>
#include <vector>

namespace uocr::download {

struct HfFileSpec {
  std::string file_id;
  std::string file_name;
  std::filesystem::path destination;
};

struct HfDownloadOptions {
  std::string repo_id;
  std::string revision = "main";
  std::string token;
  std::string user_agent = "uocr-workbench";
  bool force = false;
  std::atomic_bool* cancel_requested = nullptr;
};

struct HfDownloadProgress {
  std::string phase;
  std::string file_id;
  std::string file_name;
  std::string message;
  std::uint64_t file_downloaded_bytes = 0;
  std::uint64_t file_total_bytes = 0;
  std::uint64_t overall_downloaded_bytes = 0;
  std::uint64_t overall_total_bytes = 0;
  double file_percent = 0.0;
  double overall_percent = 0.0;
  double bytes_per_second = 0.0;
  double eta_seconds = -1.0;
};

class HfDownloadException : public std::runtime_error {
 public:
  HfDownloadException(std::string message, bool retryable, int http_status = 0);

  [[nodiscard]] bool retryable() const;
  [[nodiscard]] int http_status() const;

 private:
  bool retryable_;
  int http_status_;
};

using HfDownloadProgressCallback = std::function<void(const HfDownloadProgress&)>;

class HuggingFaceDownloader {
 public:
  void download_files(const std::vector<HfFileSpec>& files,
                      const HfDownloadOptions& options,
                      const HfDownloadProgressCallback& progress) const;
};

}  // namespace uocr::download
