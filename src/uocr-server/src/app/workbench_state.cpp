#include "workbench_state.hpp"

#include <algorithm>
#include <cctype>
#include <chrono>
#include <iomanip>
#include <sstream>
#include <utility>

#include "uocr/render/png_dimensions.hpp"

namespace uocr::server {
namespace {

bool has_extension(const std::filesystem::path& path, std::string_view expected) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == expected;
}

bool is_image_file(const std::filesystem::path& path) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == ".bmp" || ext == ".jpeg" || ext == ".jpg" || ext == ".png" || ext == ".tif" ||
         ext == ".tiff" || ext == ".webp";
}

int page_count_for(const WorkbenchService::Impl::DocumentState& document) {
  return document.pages.empty() ? 1 : static_cast<int>(document.pages.size());
}

std::size_t region_count_for(const WorkbenchService::Impl::DocumentState& document) {
  if (document.pages.empty()) {
    return document.boxes.size();
  }
  std::size_t count = 0;
  for (const auto& page : document.pages) {
    count += page.boxes.size();
  }
  return count;
}

}  // namespace

Json::Value error_json(const std::string& message) {
  Json::Value payload;
  payload["error"] = message;
  return payload;
}

std::string lower(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return value;
}

std::string now_id() {
  const auto now = std::chrono::system_clock::now().time_since_epoch();
  return "run-" + std::to_string(std::chrono::duration_cast<std::chrono::milliseconds>(now).count());
}

std::string stable_hash(const DiscoveredFile& file) {
  std::uint64_t hash = 1469598103934665603ULL;
  auto mix = [&hash](std::string_view text) {
    for (const unsigned char ch : text) {
      hash ^= ch;
      hash *= 1099511628211ULL;
    }
  };
  mix(file.absolute_path.generic_string());
  mix(std::to_string(file.size_bytes));
  std::ostringstream out;
  out << std::hex << std::setw(16) << std::setfill('0') << hash;
  return out.str();
}

WorkbenchService::Impl::Impl(std::filesystem::path root, std::shared_ptr<AppLogger> app_logger)
    : app_root(std::move(root)), logger(std::move(app_logger)) {}

std::filesystem::path WorkbenchService::Impl::model_path() const {
  return app_root / "models" / std::string(kModelFile);
}

std::filesystem::path WorkbenchService::Impl::mmproj_path() const {
  return app_root / "models" / std::string(kMmprojFile);
}

std::filesystem::path WorkbenchService::Impl::ffi_path() const {
#ifdef _WIN32
  return app_root / "thirdparty" / "uocr-runtime" / "windows-x86_64-cuda13" / "bin" / "uocr-ffi.dll";
#else
  return app_root / "thirdparty" / "uocr-runtime" / "linux-x86_64-cuda13" / "bin" / "libuocr-ffi.so";
#endif
}

bool WorkbenchService::Impl::model_ready() const {
  return std::filesystem::exists(model_path()) && std::filesystem::exists(mmproj_path());
}

Json::Value WorkbenchService::Impl::model_record() const {
  Json::Value item;
  item["model_id"] = "unlimited-ocr-q4-k-m";
  item["display_name"] = "Unlimited-OCR Q4_K_M";
  item["local_path"] = (app_root / "models").string();
  item["model_file"] = std::string(kModelFile);
  item["mmproj_file"] = std::string(kMmprojFile);
  const auto ready = model_ready();
  item["status"] = model.downloading ? "downloading" : (ready ? "downloaded" : "missing");
  if (!ready && !model.error.empty()) {
    item["status"] = "error";
    item["error"] = model.error;
  }
  item["current_file"] = model.current_file.empty() ? Json::Value(Json::nullValue) : Json::Value(model.current_file);
  item["status_message"] =
      model.status_message.empty() ? Json::Value(Json::nullValue) : Json::Value(model.status_message);
  item["downloaded_bytes"] = static_cast<Json::UInt64>(model.downloaded_bytes);
  item["total_bytes"] = model.total_bytes == 0 ? Json::Value(Json::nullValue)
                                               : Json::Value(static_cast<Json::UInt64>(model.total_bytes));

  std::uintmax_t size = 0;
  std::error_code error;
  if (std::filesystem::exists(model_path())) {
    size += std::filesystem::file_size(model_path(), error);
  }
  if (std::filesystem::exists(mmproj_path())) {
    size += std::filesystem::file_size(mmproj_path(), error);
  }
  item["size_bytes"] = static_cast<Json::UInt64>(size);
  return item;
}

bool WorkbenchService::Impl::is_image_document(const DocumentState& document) const {
  return is_image_file(document.absolute_path);
}

bool WorkbenchService::Impl::is_pdf_document(const DocumentState& document) const {
  return has_extension(document.absolute_path, ".pdf");
}

Json::Value WorkbenchService::Impl::run_record(const RunState& run) const {
  Json::Value value;
  value["run_id"] = run.run_id;
  value["root_path"] = run.root_path;
  value["status"] = run.status;
  value["queued_files"] = run.queued_files;
  value["processed_pages"] = run.processed_pages;
  value["total_pages"] = run.total_pages;
  value["error"] = run.error.empty() ? Json::Value(Json::nullValue) : Json::Value(run.error);
  return value;
}

Json::Value WorkbenchService::Impl::document_summary(const DocumentState& document) const {
  Json::Value value;
  value["file_hash"] = document.file_hash;
  value["display_name"] = document.relative_path.filename().string();
  value["relative_path"] = document.relative_path.generic_string();
  value["status"] = document.status;
  value["page_count"] = page_count_for(document);
  value["regions"] = static_cast<Json::UInt64>(region_count_for(document));
  if (!document.error.empty()) {
    value["error"] = document.error;
  }
  return value;
}

}  // namespace uocr::server
