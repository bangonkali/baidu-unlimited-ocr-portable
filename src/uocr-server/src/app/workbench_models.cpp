#include "workbench_state.hpp"

#include <filesystem>
#include <string>
#include <thread>

#include "uocr/app/app_logger.hpp"
#include "workbench_download.hpp"

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

}  // namespace

void WorkbenchService::Impl::start_download() {
  {
    std::scoped_lock lock(mutex);
    if (model.downloading || model_ready()) {
      return;
    }
    model.downloading = true;
    model.error.clear();
    model.status = "downloading";
    model.status_message = "Starting model download from Hugging Face";
    model.downloaded_bytes = 0;
    model.total_bytes = 0;
  }

  log_info(logger, "models", "model download requested");
  std::thread([shared = shared_from_this()]() {
    try {
      auto progress = [shared](const std::string& file_name, const DownloadProgress& update) {
        std::scoped_lock lock(shared->mutex);
        shared->model.current_file = file_name;
        shared->model.downloaded_bytes = update.downloaded_bytes;
        shared->model.total_bytes = update.total_bytes;
        shared->model.status_message = "Downloading " + file_name;
      };

      if (!std::filesystem::exists(shared->model_path())) {
        log_info(shared->logger, "models", "downloading " + std::string(kModelFile));
        download_to_file(std::string(kModelRepo) + std::string(kModelFile) + "?download=true", shared->model_path(),
                         [progress](const DownloadProgress& update) { progress(std::string(kModelFile), update); });
      }
      if (!std::filesystem::exists(shared->mmproj_path())) {
        log_info(shared->logger, "models", "downloading " + std::string(kMmprojFile));
        download_to_file(std::string(kModelRepo) + std::string(kMmprojFile) + "?download=true", shared->mmproj_path(),
                         [progress](const DownloadProgress& update) { progress(std::string(kMmprojFile), update); });
      }

      std::scoped_lock lock(shared->mutex);
      shared->model.downloading = false;
      shared->model.status = "downloaded";
      shared->model.current_file.clear();
      shared->model.status_message = "Models are ready";
      log_info(shared->logger, "models", "model download completed");
    } catch (const std::exception& error) {
      std::scoped_lock lock(shared->mutex);
      shared->model.downloading = false;
      shared->model.status = "error";
      shared->model.error = error.what();
      shared->model.status_message = "Model download failed";
      log_error(shared->logger, "models", std::string("model download failed: ") + error.what());
    }
  }).detach();
}

}  // namespace uocr::server
