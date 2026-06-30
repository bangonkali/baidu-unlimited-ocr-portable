#pragma once

#include "uocr/app/workbench_service.hpp"

#include <atomic>
#include <filesystem>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <string_view>
#include <vector>

#include "uocr/core/model_catalog.hpp"
#include "uocr/core/runtime_catalog.hpp"
#include "uocr/core/types.hpp"
#include "uocr/fs/file_scanner.hpp"
#include "uocr/storage/workbench_repository.hpp"

namespace uocr::server {

struct WorkbenchService::Impl : public std::enable_shared_from_this<WorkbenchService::Impl> {
  struct ModelState {
    struct File {
      std::string file_id;
      std::string file_name;
      std::filesystem::path local_path;
      std::string status = "missing";
      std::string error;
      std::uint64_t downloaded_bytes = 0;
      std::uint64_t total_bytes = 0;
      double percent = 0.0;
      double bytes_per_second = 0.0;
      double eta_seconds = -1.0;
    };

    std::string status = "missing";
    std::string error;
    std::string current_file;
    std::string status_message;
    std::string auth_source;
    std::string last_event_at;
    std::uint64_t downloaded_bytes = 0;
    std::uint64_t total_bytes = 0;
    double overall_percent = 0.0;
    double bytes_per_second = 0.0;
    double eta_seconds = -1.0;
    bool downloading = false;
    bool cancel_requested = false;
    bool auth_available = false;
    std::vector<File> files;
  };

  struct PageState {
    int page_no = 1;
    std::filesystem::path image_path;
    int width_px = 0;
    int height_px = 0;
    int dpi = 200;
    std::string status = "queued";
    std::string error;
    std::string raw_text;
    std::string cleaned_text;
    std::vector<OverlayBox> boxes;
    std::vector<TextRegionSpan> spans;
  };

  struct DocumentState {
    std::string file_hash;
    std::filesystem::path absolute_path;
    std::filesystem::path relative_path;
    std::string status = "queued";
    std::string error;
    std::string raw_text;
    std::string cleaned_text;
    std::vector<OverlayBox> boxes;
    std::vector<TextRegionSpan> spans;
    std::vector<PageState> pages;
  };

  struct RunState {
    std::string run_id;
    std::string root_path;
    std::string status = "queued";
    std::string error;
    int queued_files = 0;
    int processed_pages = 0;
    int total_pages = 0;
    bool cancel_requested = false;
    std::string profile_id = "experimental-exact-prefill-q4";
    std::string engine_id = "unlimited-ocr";
    std::string model_id = std::string(default_model_id());
    std::string runtime_id;
    std::vector<std::string> file_hashes;
  };

  Impl(std::filesystem::path root, std::shared_ptr<AppLogger> app_logger);

  std::filesystem::path model_path(std::string_view model_id) const;
  std::filesystem::path mmproj_path() const;
  RuntimeVariant selected_runtime() const;
  std::vector<RuntimeVariant> runtime_variants() const;
  std::vector<ModelState::File> model_files(const ModelCatalogEntry& entry) const;
  bool model_ready(std::string_view model_id) const;

  Json::Value model_record(const ModelCatalogEntry& entry) const;
  Json::Value model_event(std::string_view model_id) const;
  bool model_downloading(std::string_view model_id) const;
  bool any_model_downloading() const;
  Json::Value status_record() const;
  Json::Value workbench_ui_settings() const;
  Json::Value run_record(const RunState& run) const;
  Json::Value document_summary(const DocumentState& document) const;
  Json::Value document_page_record(const DocumentState& document, const PageState& page) const;
  Json::Value document_regions_record(const DocumentState& document) const;
  Json::Value document_text_record(const DocumentState& document) const;
  bool is_image_document(const DocumentState& document) const;
  bool is_pdf_document(const DocumentState& document) const;
  std::string document_status_for(const DocumentState& document) const;
  void apply_region_content(PageState& page) const;
  void refresh_document_aggregate(DocumentState& document) const;
  std::vector<PageState> prepare_pages(const DiscoveredFile& file, const std::string& file_hash) const;

  void load_persisted_snapshot();
  void publish_event(std::string_view type, const Json::Value& payload) const;
  void publish_status_changed() const;
  void persist_run(const RunState& run) const;
  void persist_document(const DocumentState& document, std::string_view root_path) const;
  void persist_page(const std::string& file_hash, const PageState& page) const;
  void persist_page_ocr(const std::string& file_hash,
                        const PageState& page,
                        std::string_view profile_id) const;
  void persist_work_unit(const std::string& run_id,
                         const std::string& file_hash,
                         int page_no,
                         std::string_view status,
                         int attempts,
                         std::string_view error) const;
  void persist_diagnostic(const std::string& run_id, std::string_view level, std::string_view message) const;
  void persist_selected_model() const;
  void persist_selected_runtime() const;
  void persist_selected_profile() const;
  bool update_workbench_ui_settings(const Json::Value& patch, std::string& error) const;
  bool select_runtime(std::string_view runtime_id);
  void start_download(std::string model_id, bool force);
  void cancel_download(std::string_view model_id);
  void start_run(const std::string& run_id,
                 std::vector<DiscoveredFile> files,
                 std::string profile_id,
                 std::string model_id);
  void fail_run(const std::string& run_id, const std::string& message);
  void process_run(const std::string& run_id,
                   const std::vector<DiscoveredFile>& files,
                   const std::string& profile_id,
                   const std::string& model_id);

  std::filesystem::path app_root;
  std::shared_ptr<AppLogger> logger;
  std::shared_ptr<uocr::storage::WorkbenchRepository> repository;
  std::atomic_bool model_cancel_requested{false};
  mutable std::mutex mutex;
  std::string selected_model_id = std::string(default_model_id());
  std::string selected_profile_id = "experimental-exact-prefill-q4";
  RuntimeHardwareProbe hardware_probe;
  std::string selected_runtime_id;
  std::string active_download_model_id;
  std::map<std::string, ModelState> models;
  std::map<std::string, RunState> runs;
  std::map<std::string, DocumentState> documents;
};

Json::Value error_json(const std::string& message);
std::string lower(std::string value);
std::string now_id();
std::string stable_hash(const DiscoveredFile& file);

}  // namespace uocr::server
