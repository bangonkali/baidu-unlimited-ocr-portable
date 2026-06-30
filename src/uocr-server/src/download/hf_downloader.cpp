#include "uocr/download/hf_downloader.hpp"

#include <algorithm>
#include <chrono>
#include <thread>
#include <utility>

#include "uocr/download/download_progress.hpp"
#include "hf_transfer.hpp"

namespace uocr::download {
namespace {

constexpr int kMaxAttempts = 3;

void sleep_before_retry(int attempt) {
  const int seconds[] = {2, 5, 10};
  std::this_thread::sleep_for(std::chrono::seconds(seconds[std::clamp(attempt - 1, 0, 2)]));
}

HfDownloadProgress skipped_progress(const detail::PreparedFile& file,
                                    std::uint64_t local_size,
                                    std::uint64_t completed_before,
                                    std::uint64_t overall_total) {
  HfDownloadProgress update;
  update.phase = "downloaded";
  update.file_id = file.spec.file_id;
  update.file_name = file.spec.file_name;
  update.message = "Using existing " + file.spec.file_name;
  update.file_downloaded_bytes = local_size;
  update.file_total_bytes = file.size == 0 ? local_size : file.size;
  update.overall_downloaded_bytes = completed_before;
  update.overall_total_bytes = overall_total;
  update.file_percent = 100.0;
  update.overall_percent = percent_complete(completed_before, overall_total);
  return update;
}

HfDownloadProgress verified_progress(const detail::PreparedFile& file,
                                     std::uint64_t completed_before,
                                     std::uint64_t overall_total) {
  HfDownloadProgress update;
  update.phase = "verified";
  update.file_id = file.spec.file_id;
  update.file_name = file.spec.file_name;
  update.message = "Verified " + file.spec.file_name;
  update.file_downloaded_bytes = detail::existing_size(file.spec.destination);
  update.file_total_bytes = file.size == 0 ? update.file_downloaded_bytes : file.size;
  update.overall_downloaded_bytes = completed_before;
  update.overall_total_bytes = overall_total;
  update.file_percent = 100.0;
  update.overall_percent = percent_complete(completed_before, overall_total);
  return update;
}

HfDownloadProgress verifying_progress(const detail::PreparedFile& file,
                                      std::uint64_t completed_before,
                                      std::uint64_t overall_total) {
  HfDownloadProgress update;
  update.phase = "verifying";
  update.file_id = file.spec.file_id;
  update.file_name = file.spec.file_name;
  update.message = file.sha256.empty() ? "Checking file size for " + file.spec.file_name
                                       : "Verifying SHA256 for " + file.spec.file_name;
  update.file_downloaded_bytes = detail::existing_size(file.spec.destination);
  update.file_total_bytes = file.size == 0 ? update.file_downloaded_bytes : file.size;
  update.overall_downloaded_bytes = completed_before + update.file_downloaded_bytes;
  update.overall_total_bytes = overall_total;
  update.file_percent = percent_complete(update.file_downloaded_bytes, update.file_total_bytes);
  update.overall_percent = percent_complete(update.overall_downloaded_bytes, overall_total);
  return update;
}

HfDownloadProgress retry_progress(const detail::PreparedFile& file,
                                  const HfDownloadException& error,
                                  std::uint64_t completed_before,
                                  std::uint64_t overall_total) {
  HfDownloadProgress update;
  update.phase = "retrying";
  update.file_id = file.spec.file_id;
  update.file_name = file.spec.file_name;
  update.message = "Retrying " + file.spec.file_name + " after " + error.what();
  update.overall_total_bytes = overall_total;
  update.overall_downloaded_bytes = completed_before;
  return update;
}

}  // namespace

HfDownloadException::HfDownloadException(std::string message, bool retryable, int http_status)
    : std::runtime_error(std::move(message)), retryable_(retryable), http_status_(http_status) {}

bool HfDownloadException::retryable() const {
  return retryable_;
}

int HfDownloadException::http_status() const {
  return http_status_;
}

void HuggingFaceDownloader::download_files(const std::vector<HfFileSpec>& files,
                                           const HfDownloadOptions& options,
                                           const HfDownloadProgressCallback& progress) const {
  detail::initialize_curl_once();
  std::vector<detail::PreparedFile> prepared_files;
  prepared_files.reserve(files.size());

  for (const auto& file : files) {
    HfDownloadProgress update;
    update.phase = "metadata";
    update.file_id = file.file_id;
    update.file_name = file.file_name;
    update.message = "Checking Hugging Face metadata for " + file.file_name;
    progress(update);
    prepared_files.push_back(detail::prepare_file(file, options));
  }

  std::uint64_t overall_total = 0;
  for (const auto& file : prepared_files) {
    overall_total += file.size;
  }

  std::uint64_t completed_before = 0;
  for (const auto& file : prepared_files) {
    const auto local_size = detail::existing_size(file.spec.destination);
    if (!options.force && local_size > 0 && (file.size == 0 || local_size == file.size)) {
      progress(verifying_progress(file, completed_before, overall_total));
      detail::validate_download(file);
      completed_before += file.size == 0 ? local_size : file.size;
      progress(skipped_progress(file, local_size, completed_before, overall_total));
      continue;
    }

    for (int attempt = 1; attempt <= kMaxAttempts; ++attempt) {
      try {
        detail::download_prepared_file(file, options, completed_before, overall_total, progress);
        completed_before += file.size == 0 ? detail::existing_size(file.spec.destination) : file.size;
        progress(verified_progress(file, completed_before, overall_total));
        break;
      } catch (const HfDownloadException& error) {
        if (!error.retryable() || attempt == kMaxAttempts) {
          throw;
        }
        progress(retry_progress(file, error, completed_before, overall_total));
        sleep_before_retry(attempt);
      }
    }
  }

  HfDownloadProgress update;
  update.phase = "completed";
  update.message = "Model files are ready";
  update.overall_downloaded_bytes = overall_total;
  update.overall_total_bytes = overall_total;
  update.file_percent = 100.0;
  update.overall_percent = 100.0;
  progress(update);
}

}  // namespace uocr::download
