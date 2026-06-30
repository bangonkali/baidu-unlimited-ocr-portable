#pragma once

#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

#include "uocr/core/types.hpp"

namespace uocr::storage {

struct StoredPage {
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

struct StoredDocument {
  std::string file_hash;
  std::filesystem::path absolute_path;
  std::filesystem::path relative_path;
  std::string status = "queued";
  std::string error;
  std::uint64_t size_bytes = 0;
  int page_count = 1;
  std::vector<StoredPage> pages;
};

struct StoredRun {
  std::string run_id;
  std::string root_path;
  std::string status = "queued";
  std::string error;
  std::string profile_id = "experimental-exact-prefill-q4";
  std::string engine_id = "unlimited-ocr";
  std::string model_id = "unlimited-ocr-q4-k-m";
  std::string runtime_id;
  int queued_files = 0;
  int processed_pages = 0;
  int total_pages = 0;
  std::vector<std::string> file_hashes;
};

struct WorkbenchSnapshot {
  std::vector<StoredRun> runs;
  std::vector<StoredDocument> documents;
};

class WorkbenchRepository {
 public:
  struct Impl;

  explicit WorkbenchRepository(std::filesystem::path database_path);
  ~WorkbenchRepository();
  WorkbenchRepository(const WorkbenchRepository&) = delete;
  WorkbenchRepository& operator=(const WorkbenchRepository&) = delete;
  WorkbenchRepository(WorkbenchRepository&&) noexcept;
  WorkbenchRepository& operator=(WorkbenchRepository&&) noexcept;

  const std::filesystem::path& database_path() const;
  WorkbenchSnapshot load_snapshot() const;
  std::vector<std::string> search_document_hashes(std::string_view query, std::size_t limit) const;
  std::string setting_string(std::string_view key, std::string_view fallback) const;

  void upsert_run(const StoredRun& run);
  void put_setting_string(std::string_view key, std::string_view value);
  void upsert_document(const StoredDocument& document, std::string_view root_path);
  void upsert_page(const std::string& file_hash, const StoredPage& page);
  void replace_page_ocr(const std::string& file_hash,
                        const StoredPage& page,
                        std::string_view engine_id,
                        std::string_view profile_id);
  void upsert_work_unit(const std::string& run_id,
                        const std::string& file_hash,
                        int page_no,
                        std::string_view status,
                        int attempts,
                        std::string_view error);
  void append_diagnostic_event(const std::string& run_id,
                               std::string_view level,
                               std::string_view message);

 private:
  std::unique_ptr<Impl> impl_;
};

}  // namespace uocr::storage
