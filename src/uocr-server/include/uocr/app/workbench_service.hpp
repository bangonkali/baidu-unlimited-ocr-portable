#pragma once

#include <filesystem>
#include <memory>
#include <optional>
#include <string>

#include <json/json.h>

namespace uocr::server {

class AppLogger;

class WorkbenchService {
 public:
  WorkbenchService(std::filesystem::path app_root, std::shared_ptr<AppLogger> logger);

  Json::Value status() const;
  Json::Value models() const;
  Json::Value select_model(const std::string& model_id);
  Json::Value start_model_download(const std::string& model_id, bool force);
  Json::Value cancel_model_download(const std::string& model_id);
  Json::Value model_download_event(const std::string& model_id) const;
  bool model_downloading(const std::string& model_id) const;

  Json::Value settings() const;
  Json::Value update_settings(const Json::Value& request);
  Json::Value start_ingest(const Json::Value& request);
  Json::Value list_runs() const;
  Json::Value get_run(const std::string& run_id) const;
  Json::Value run_metrics(const std::string& run_id) const;
  Json::Value recent_metrics(std::size_t limit) const;
  Json::Value run_command(const std::string& run_id, const std::string& command);
  std::string run_event_stream(const std::string& run_id) const;

  Json::Value list_documents(const std::string& query) const;
  Json::Value get_document(const std::string& file_hash) const;
  Json::Value document_regions(const std::string& file_hash) const;
  Json::Value document_text(const std::string& file_hash) const;
  Json::Value document_preview_images(const std::string& file_hash) const;
  std::optional<std::filesystem::path> document_preview_image(const std::string& file_hash,
                                                              const std::string& variant,
                                                              int page_no) const;

  struct Impl;

 private:
  std::shared_ptr<Impl> impl_;
};

}  // namespace uocr::server
