#include "hf_transfer.hpp"

#include <curl/curl.h>

#include <chrono>
#include <exception>
#include <fstream>

#include "uocr/download/download_progress.hpp"
#include "uocr/download/sha256.hpp"

namespace uocr::download::detail {
namespace {

struct CurlHandle {
  CurlHandle() : handle(curl_easy_init()) {
    if (handle == nullptr) {
      throw HfDownloadException("could not initialize libcurl", true);
    }
  }
  ~CurlHandle() { curl_easy_cleanup(handle); }
  CURL* handle;
};

struct ProgressContext {
  PreparedFile file;
  HfDownloadProgressCallback callback;
  std::atomic_bool* cancel_requested = nullptr;
  std::uint64_t resume_offset = 0;
  std::uint64_t completed_before = 0;
  std::uint64_t overall_total = 0;
  std::chrono::steady_clock::time_point started_at = std::chrono::steady_clock::now();
  std::chrono::steady_clock::time_point last_emit = started_at;
  std::exception_ptr callback_error;
};

curl_slist* build_headers(const std::string& token) {
  curl_slist* headers = nullptr;
  if (!token.empty()) {
    const auto auth = "Authorization: Bearer " + token;
    headers = curl_slist_append(headers, auth.c_str());
  }
  return headers;
}

void set_common_options(CURL* curl, const std::string& url, const std::string& user_agent) {
  curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
  curl_easy_setopt(curl, CURLOPT_USERAGENT, user_agent.c_str());
  curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1L);
  curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT, 30L);
  curl_easy_setopt(curl, CURLOPT_LOW_SPEED_LIMIT, 1024L);
  curl_easy_setopt(curl, CURLOPT_LOW_SPEED_TIME, 60L);
}

size_t write_file_callback(char* buffer, size_t size, size_t items, void* user_data) {
  auto* output = static_cast<std::ofstream*>(user_data);
  const auto bytes = size * items;
  output->write(buffer, static_cast<std::streamsize>(bytes));
  return output->good() ? bytes : 0;
}

void emit_progress(ProgressContext& context, curl_off_t downloaded_now, bool force) {
  const auto now = std::chrono::steady_clock::now();
  if (!force && now - context.last_emit < std::chrono::milliseconds(250)) {
    return;
  }
  context.last_emit = now;
  const auto file_downloaded = context.resume_offset + static_cast<std::uint64_t>(downloaded_now);
  const auto file_total = context.file.size == 0 ? file_downloaded : context.file.size;
  const auto speed =
      transfer_rate_bytes_per_second(static_cast<std::uint64_t>(downloaded_now), now - context.started_at);
  const auto eta = eta_seconds(file_downloaded, file_total, speed);
  HfDownloadProgress update;
  update.phase = "downloading";
  update.file_id = context.file.spec.file_id;
  update.file_name = context.file.spec.file_name;
  update.message = "Downloading " + context.file.spec.file_name;
  update.file_downloaded_bytes = file_downloaded;
  update.file_total_bytes = file_total;
  update.overall_downloaded_bytes = context.completed_before + file_downloaded;
  update.overall_total_bytes = context.overall_total;
  update.file_percent = percent_complete(file_downloaded, file_total);
  update.overall_percent = percent_complete(update.overall_downloaded_bytes, update.overall_total_bytes);
  update.bytes_per_second = speed;
  update.eta_seconds = eta.value_or(-1.0);
  context.callback(update);
}

int progress_callback(void* user_data, curl_off_t, curl_off_t downloaded_now, curl_off_t, curl_off_t) {
  auto* context = static_cast<ProgressContext*>(user_data);
  if (context->cancel_requested != nullptr && context->cancel_requested->load()) {
    return 1;
  }
  try {
    emit_progress(*context, downloaded_now, false);
  } catch (...) {
    context->callback_error = std::current_exception();
    return 1;
  }
  return 0;
}

void emit_verifying_progress(const PreparedFile& file,
                             std::uint64_t completed_before,
                             std::uint64_t overall_total,
                             const HfDownloadProgressCallback& progress) {
  const auto file_size = existing_size(file.spec.destination);
  HfDownloadProgress update;
  update.phase = "verifying";
  update.file_id = file.spec.file_id;
  update.file_name = file.spec.file_name;
  update.message = file.sha256.empty() ? "Checking file size for " + file.spec.file_name
                                       : "Verifying SHA256 for " + file.spec.file_name;
  update.file_downloaded_bytes = file_size;
  update.file_total_bytes = file.size == 0 ? file_size : file.size;
  update.overall_downloaded_bytes = completed_before + file_size;
  update.overall_total_bytes = overall_total;
  update.file_percent = percent_complete(file_size, update.file_total_bytes);
  update.overall_percent = percent_complete(update.overall_downloaded_bytes, overall_total);
  progress(update);
}

}  // namespace

std::uint64_t existing_size(const std::filesystem::path& path) {
  std::error_code error;
  return std::filesystem::exists(path, error) ? static_cast<std::uint64_t>(std::filesystem::file_size(path, error)) : 0;
}

void validate_download(const PreparedFile& file) {
  if (file.size != 0 && std::filesystem::file_size(file.spec.destination) != file.size) {
    throw HfDownloadException("downloaded file size mismatch for " + file.spec.file_name, false);
  }
  if (!file.sha256.empty() && sha256_file(file.spec.destination) != file.sha256) {
    std::error_code error;
    std::filesystem::remove(file.spec.destination, error);
    throw HfDownloadException("SHA256 mismatch for " + file.spec.file_name, false);
  }
}

void download_prepared_file(const PreparedFile& file,
                            const HfDownloadOptions& options,
                            std::uint64_t completed_before,
                            std::uint64_t overall_total,
                            const HfDownloadProgressCallback& progress) {
  std::filesystem::create_directories(file.spec.destination.parent_path());
  const auto temp = file.spec.destination.string() + ".download";
  if (options.force) {
    std::error_code error;
    std::filesystem::remove(file.spec.destination, error);
    std::filesystem::remove(temp, error);
  }
  auto resume_offset = existing_size(temp);
  if (file.size != 0 && resume_offset > file.size) {
    std::filesystem::remove(temp);
    resume_offset = 0;
  }
  std::ofstream output(temp, std::ios::binary | std::ios::app);
  if (!output) {
    throw HfDownloadException("could not open temporary download file: " + temp, false);
  }
  ProgressContext progress_context{.file = file,
                                   .callback = progress,
                                   .cancel_requested = options.cancel_requested,
                                   .resume_offset = resume_offset,
                                   .completed_before = completed_before,
                                   .overall_total = overall_total};
  CurlHandle curl;
  auto* request_headers = file.send_auth ? build_headers(options.token) : nullptr;
  set_common_options(curl.handle, file.url, options.user_agent);
  curl_easy_setopt(curl.handle, CURLOPT_FOLLOWLOCATION, file.send_auth ? 0L : 1L);
  curl_easy_setopt(curl.handle, CURLOPT_WRITEFUNCTION, write_file_callback);
  curl_easy_setopt(curl.handle, CURLOPT_WRITEDATA, &output);
  curl_easy_setopt(curl.handle, CURLOPT_NOPROGRESS, 0L);
  curl_easy_setopt(curl.handle, CURLOPT_XFERINFOFUNCTION, progress_callback);
  curl_easy_setopt(curl.handle, CURLOPT_XFERINFODATA, &progress_context);
  curl_easy_setopt(curl.handle, CURLOPT_HTTPHEADER, request_headers);
  if (resume_offset > 0) {
    curl_easy_setopt(curl.handle, CURLOPT_RESUME_FROM_LARGE, static_cast<curl_off_t>(resume_offset));
  }
  const auto result = curl_easy_perform(curl.handle);
  curl_slist_free_all(request_headers);
  output.close();
  if (progress_context.callback_error) {
    std::rethrow_exception(progress_context.callback_error);
  }
  if (result == CURLE_ABORTED_BY_CALLBACK) {
    throw HfDownloadException("download cancelled", false);
  }
  if (result != CURLE_OK) {
    throw HfDownloadException("download failed: " + std::string(curl_easy_strerror(result)), true);
  }
  long response_code = 0;
  curl_easy_getinfo(curl.handle, CURLINFO_RESPONSE_CODE, &response_code);
  if (response_code >= 400) {
    const bool retryable = response_code >= 500;
    throw HfDownloadException("download returned HTTP " + std::to_string(response_code), retryable,
                              static_cast<int>(response_code));
  }
  emit_progress(progress_context, static_cast<curl_off_t>(existing_size(temp) - resume_offset), true);
  std::error_code error;
  std::filesystem::rename(temp, file.spec.destination, error);
  if (error) {
    std::filesystem::remove(file.spec.destination, error);
    std::filesystem::rename(temp, file.spec.destination, error);
  }
  if (error) {
    throw HfDownloadException("could not finalize " + file.spec.file_name, false);
  }
  emit_verifying_progress(file, completed_before, overall_total, progress);
  validate_download(file);
}

}  // namespace uocr::download::detail
