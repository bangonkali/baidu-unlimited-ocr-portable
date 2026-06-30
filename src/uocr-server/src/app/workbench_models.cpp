#include "workbench_state.hpp"

#include <chrono>
#include <iomanip>
#include <filesystem>
#include <memory>
#include <sstream>
#include <string>
#include <thread>

#include "uocr/app/app_logger.hpp"
#include "uocr/download/hf_auth.hpp"
#include "uocr/download/hf_downloader.hpp"

namespace uocr::server {
namespace {

void log_info(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->info(component, message);
  }
}

void log_error(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->error(component, message);
  }
}

std::string utc_timestamp() {
  const auto now = std::chrono::system_clock::now();
  const auto time = std::chrono::system_clock::to_time_t(now);
  std::tm utc{};
#ifdef _WIN32
  gmtime_s(&utc, &time);
#else
  gmtime_r(&time, &utc);
#endif
  std::ostringstream stream;
  stream << std::put_time(&utc, "%Y-%m-%dT%H:%M:%SZ");
  return stream.str();
}

std::string format_mib(double bytes) {
  std::ostringstream stream;
  stream << std::fixed << std::setprecision(1) << (bytes / 1024.0 / 1024.0);
  return stream.str();
}

std::string progress_message(const uocr::download::HfDownloadProgress& update) {
  if (update.phase != "downloading") {
    return update.message;
  }
  return update.file_name + " " + format_mib(static_cast<double>(update.file_downloaded_bytes)) + " / " +
         format_mib(static_cast<double>(update.file_total_bytes)) + " MiB at " +
         format_mib(update.bytes_per_second) + " MiB/s";
}

void apply_progress_locked(WorkbenchService::Impl& state,
                           std::string_view model_id,
                           const uocr::download::HfDownloadProgress& update) {
  auto& model = state.models[std::string(model_id)];
  if (!update.file_id.empty()) {
    for (auto& file : model.files) {
      if (file.file_id != update.file_id) {
        continue;
      }
      file.status = update.phase == "verified" ? "downloaded" : update.phase;
      file.error.clear();
      file.downloaded_bytes = update.file_downloaded_bytes;
      file.total_bytes = update.file_total_bytes;
      file.percent = update.file_percent;
      file.bytes_per_second = update.bytes_per_second;
      file.eta_seconds = update.eta_seconds;
      break;
    }
  }
  model.current_file = update.file_name;
  model.status = update.phase == "completed" ? "downloaded" : "downloading";
  model.status_message = progress_message(update);
  model.downloaded_bytes = update.overall_downloaded_bytes;
  model.total_bytes = update.overall_total_bytes;
  model.overall_percent = update.overall_percent;
  model.bytes_per_second = update.bytes_per_second;
  model.eta_seconds = update.eta_seconds;
  model.last_event_at = utc_timestamp();
}

std::vector<uocr::download::HfFileSpec> download_specs(
    const std::vector<WorkbenchService::Impl::ModelState::File>& files) {
  std::vector<uocr::download::HfFileSpec> specs;
  specs.reserve(files.size());
  for (const auto& file : files) {
    specs.push_back({.file_id = file.file_id, .file_name = file.file_name, .destination = file.local_path});
  }
  return specs;
}

void finish_download_failure(const std::shared_ptr<WorkbenchService::Impl>& shared,
                             std::string_view model_id,
                             const std::string& message) {
  Json::Value event;
  const bool cancelled = shared->model_cancel_requested.load();
  {
    std::scoped_lock lock(shared->mutex);
    auto& model = shared->models[std::string(model_id)];
    model.downloading = false;
    model.cancel_requested = false;
    model.status = cancelled ? "cancelled" : "error";
    model.error = cancelled ? std::string() : message;
    model.status_message = cancelled ? "Model download cancelled; retry will resume partial files"
                                     : "Model download failed";
    model.last_event_at = utc_timestamp();
    shared->active_download_model_id.clear();
    event = shared->model_event(model_id);
  }
  shared->publish_event("model.changed", event);
  if (cancelled) {
    log_info(shared->logger, "models", "model download cancelled for " + std::string(model_id));
  } else {
    log_error(shared->logger, "models", "model download failed for " + std::string(model_id) + ": " + message);
  }
}

}  // namespace

void WorkbenchService::Impl::start_download(std::string model_id, bool force) {
  const auto* entry = find_model_catalog_entry(model_id);
  if (entry == nullptr) {
    return;
  }
  const auto auth = uocr::download::read_hf_auth_from_environment();
  Json::Value initial_event;
  {
    std::scoped_lock lock(mutex);
    auto& model = models[model_id];
    if (any_model_downloading() || (model_ready(model_id) && !force)) {
      return;
    }
    model_cancel_requested.store(false);
    model.files = model_files(*entry);
    model.downloading = true;
    model.cancel_requested = false;
    model.auth_available = auth.available();
    model.auth_source = auth.source;
    model.error.clear();
    model.status = "downloading";
    model.status_message = auth.available() ? "Starting authenticated Hugging Face download"
                                            : "Starting public Hugging Face download";
    model.downloaded_bytes = 0;
    model.total_bytes = 0;
    model.overall_percent = 0.0;
    model.bytes_per_second = 0.0;
    model.eta_seconds = -1.0;
    model.last_event_at = utc_timestamp();
    active_download_model_id = model_id;
    initial_event = model_event(model_id);
  }

  publish_event("model.changed", initial_event);
  log_info(logger, "models", "model download requested model=" + model_id +
                                 " auth=" + (auth.available() ? "env" : "none"));
  std::thread([shared = shared_from_this(), auth, force, model_id, entry = *entry]() {
    try {
      uocr::download::HuggingFaceDownloader downloader;
      uocr::download::HfDownloadOptions options;
      options.repo_id = std::string(provider_repo_id());
      options.revision = std::string(provider_revision());
      options.token = auth.token;
      options.user_agent = "uocr-workbench";
      options.force = force;
      options.cancel_requested = &shared->model_cancel_requested;

      auto last_progress_log =
          std::make_shared<std::chrono::steady_clock::time_point>(std::chrono::steady_clock::now());
      auto progress = [shared, last_progress_log, model_id](const uocr::download::HfDownloadProgress& update) {
        Json::Value event;
        {
          std::scoped_lock lock(shared->mutex);
          apply_progress_locked(*shared, model_id, update);
          event = shared->model_event(model_id);
        }
        shared->publish_event("model.changed", event);
        const auto now = std::chrono::steady_clock::now();
        if (update.phase == "downloading" && now - *last_progress_log >= std::chrono::seconds(5)) {
          *last_progress_log = now;
          log_info(shared->logger, "models", progress_message(update));
        }
        if (update.phase != "downloading" || update.file_downloaded_bytes == update.file_total_bytes) {
          log_info(shared->logger, "models", progress_message(update));
        }
      };

      downloader.download_files(download_specs(shared->model_files(entry)), options, progress);

      Json::Value event;
      {
        std::scoped_lock lock(shared->mutex);
        auto& model = shared->models[model_id];
        model.downloading = false;
        model.cancel_requested = false;
        model.status = "downloaded";
        model.current_file.clear();
        model.status_message = std::string(entry.display_name) + " is ready";
        model.overall_percent = 100.0;
        model.bytes_per_second = 0.0;
        model.eta_seconds = -1.0;
        model.last_event_at = utc_timestamp();
        shared->active_download_model_id.clear();
        event = shared->model_event(model_id);
      }
      shared->publish_event("model.changed", event);
      log_info(shared->logger, "models", "model download completed for " + model_id);
    } catch (const std::exception& error) {
      finish_download_failure(shared, model_id, error.what());
    } catch (...) {
      finish_download_failure(shared, model_id, "unknown model download failure");
    }
  }).detach();
}

void WorkbenchService::Impl::cancel_download(std::string_view model_id) {
  Json::Value event;
  {
    std::scoped_lock lock(mutex);
    auto found = models.find(std::string(model_id));
    if (found == models.end() || !found->second.downloading) {
      return;
    }
    auto& model = found->second;
    model.cancel_requested = true;
    model.status_message = "Cancelling model download";
    model.last_event_at = utc_timestamp();
    model_cancel_requested.store(true);
    event = model_event(model_id);
  }
  log_info(logger, "models", "model download cancel requested for " + std::string(model_id));
  publish_event("model.changed", event);
}

}  // namespace uocr::server
