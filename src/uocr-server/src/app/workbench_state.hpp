#pragma once

#include "uocr/app/workbench_service.hpp"

#include <filesystem>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <string_view>
#include <vector>

#include "uocr/core/types.hpp"
#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

inline constexpr std::string_view kModelRepo =
    "https://huggingface.co/sahilchachra/Unlimited-OCR-GGUF/resolve/main/";
inline constexpr std::string_view kModelFile = "Unlimited-OCR-Q4_K_M.gguf";
inline constexpr std::string_view kMmprojFile = "mmproj-Unlimited-OCR-F16.gguf";

struct WorkbenchService::Impl : public std::enable_shared_from_this<WorkbenchService::Impl> {
  struct ModelState {
    std::string status = "missing";
    std::string error;
    std::string current_file;
    std::string status_message;
    std::uint64_t downloaded_bytes = 0;
    std::uint64_t total_bytes = 0;
    bool downloading = false;
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
    std::vector<std::string> file_hashes;
  };

  Impl(std::filesystem::path root, std::shared_ptr<AppLogger> app_logger);

  std::filesystem::path model_path() const;
  std::filesystem::path mmproj_path() const;
  std::filesystem::path ffi_path() const;
  bool model_ready() const;

  Json::Value model_record() const;
  Json::Value run_record(const RunState& run) const;
  Json::Value document_summary(const DocumentState& document) const;
  bool is_image_document(const DocumentState& document) const;
  bool is_pdf_document(const DocumentState& document) const;
  std::vector<PageState> prepare_pages(const DiscoveredFile& file, const std::string& file_hash) const;

  void start_download();
  void start_run(const std::string& run_id, std::vector<DiscoveredFile> files, std::string profile_id);
  void fail_run(const std::string& run_id, const std::string& message);
  void process_run(const std::string& run_id,
                   const std::vector<DiscoveredFile>& files,
                   const std::string& profile_id);

  std::filesystem::path app_root;
  std::shared_ptr<AppLogger> logger;
  mutable std::mutex mutex;
  ModelState model;
  std::map<std::string, RunState> runs;
  std::map<std::string, DocumentState> documents;
};

Json::Value error_json(const std::string& message);
std::string lower(std::string value);
std::string now_id();
std::string stable_hash(const DiscoveredFile& file);

}  // namespace uocr::server
