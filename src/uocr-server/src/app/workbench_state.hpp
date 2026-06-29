#pragma once

#include "uocr/app/workbench_service.hpp"

#include <filesystem>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

#include "uocr/core/types.hpp"
#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

struct WorkbenchService::Impl : public std::enable_shared_from_this<WorkbenchService::Impl> {
  struct ModelState {
    std::string status = "missing";
    std::string error;
    bool downloading = false;
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

  explicit Impl(std::filesystem::path root);

  std::filesystem::path model_path() const;
  std::filesystem::path mmproj_path() const;
  std::filesystem::path ffi_path() const;
  bool model_ready() const;

  Json::Value model_record() const;
  Json::Value run_record(const RunState& run) const;
  Json::Value document_summary(const DocumentState& document) const;

  void start_download();
  void start_run(const std::string& run_id, std::vector<DiscoveredFile> files, std::string profile_id);
  void fail_run(const std::string& run_id, const std::string& message);
  void process_run(const std::string& run_id,
                   const std::vector<DiscoveredFile>& files,
                   const std::string& profile_id);

  std::filesystem::path app_root;
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
