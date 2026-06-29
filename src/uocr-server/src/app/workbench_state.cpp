#include "workbench_state.hpp"

#include <algorithm>
#include <cctype>
#include <chrono>
#include <iomanip>
#include <sstream>
#include <stdexcept>
#include <thread>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <urlmon.h>
#endif

#include "uocr/core/ocr_parser.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/ocr/unlimited_ocr_ffi_engine.hpp"

namespace uocr::server {
namespace {

constexpr const char* kModelRepo = "https://huggingface.co/sahilchachra/Unlimited-OCR-GGUF/resolve/main/";
constexpr const char* kModelFile = "Unlimited-OCR-Q4_K_M.gguf";
constexpr const char* kMmprojFile = "mmproj-Unlimited-OCR-F16.gguf";

bool is_image_file(const std::filesystem::path& path) {
  const auto ext = path.extension().string();
  return ext == ".bmp" || ext == ".jpeg" || ext == ".jpg" || ext == ".png" || ext == ".tif" ||
         ext == ".tiff" || ext == ".webp";
}

void download_to_file(const std::string& url, const std::filesystem::path& destination) {
  std::filesystem::create_directories(destination.parent_path());
  const auto temp = destination.string() + ".download";
#ifdef _WIN32
  const auto result = URLDownloadToFileA(nullptr, url.c_str(), temp.c_str(), 0, nullptr);
  if (result != S_OK) {
    throw std::runtime_error("download failed for " + url);
  }
#else
  (void)url;
  throw std::runtime_error("model download is implemented for Windows portable builds first");
#endif
  std::error_code error;
  std::filesystem::rename(temp, destination, error);
  if (error) {
    std::filesystem::remove(destination, error);
    std::filesystem::rename(temp, destination, error);
  }
  if (error) {
    throw std::runtime_error("could not finalize model download: " + destination.string());
  }
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

WorkbenchService::Impl::Impl(std::filesystem::path root) : app_root(std::move(root)) {}

std::filesystem::path WorkbenchService::Impl::model_path() const {
  return app_root / "models" / kModelFile;
}

std::filesystem::path WorkbenchService::Impl::mmproj_path() const {
  return app_root / "models" / kMmprojFile;
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
  item["model_file"] = kModelFile;
  item["mmproj_file"] = kMmprojFile;
  const auto ready = model_ready();
  item["status"] = model.downloading ? "downloading" : (ready ? "downloaded" : "missing");
  if (!ready && !model.error.empty()) {
    item["status"] = "error";
    item["error"] = model.error;
  }
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
  value["page_count"] = 1;
  value["regions"] = static_cast<Json::UInt64>(document.boxes.size());
  if (!document.error.empty()) {
    value["error"] = document.error;
  }
  return value;
}

void WorkbenchService::Impl::start_download() {
  {
    std::scoped_lock lock(mutex);
    if (model.downloading || model_ready()) {
      return;
    }
    model.downloading = true;
    model.error.clear();
    model.status = "downloading";
  }
  std::thread([shared = shared_from_this()]() {
    try {
      if (!std::filesystem::exists(shared->model_path())) {
        download_to_file(std::string(kModelRepo) + kModelFile + "?download=true", shared->model_path());
      }
      if (!std::filesystem::exists(shared->mmproj_path())) {
        download_to_file(std::string(kModelRepo) + kMmprojFile + "?download=true", shared->mmproj_path());
      }
      std::scoped_lock lock(shared->mutex);
      shared->model.downloading = false;
      shared->model.status = "downloaded";
    } catch (const std::exception& error) {
      std::scoped_lock lock(shared->mutex);
      shared->model.downloading = false;
      shared->model.status = "error";
      shared->model.error = error.what();
    }
  }).detach();
}

void WorkbenchService::Impl::start_run(std::string const& run_id,
                                       std::vector<DiscoveredFile> files,
                                       std::string profile_id) {
  std::thread([shared = shared_from_this(), run_id, files = std::move(files), profile_id = std::move(profile_id)]() {
    shared->process_run(run_id, files, profile_id);
  }).detach();
}

void WorkbenchService::Impl::fail_run(const std::string& run_id, const std::string& message) {
  std::scoped_lock lock(mutex);
  auto& run = runs[run_id];
  run.status = "failed";
  run.error = message;
  for (const auto& hash : run.file_hashes) {
    auto& document = documents[hash];
    if (document.status == "queued" || document.status == "running") {
      document.status = "failed";
      document.error = message;
    }
  }
}

void WorkbenchService::Impl::process_run(const std::string& run_id,
                                         const std::vector<DiscoveredFile>& files,
                                         const std::string& profile_id) {
  const auto* profile = find_ocr_profile(profile_id);
  profile = profile != nullptr ? profile : &default_ocr_profile();
  if (!model_ready()) {
    fail_run(run_id, "model assets are missing; use POST /api/models/unlimited-ocr-q4-k-m/download");
    return;
  }
  if (!std::filesystem::exists(ffi_path())) {
    fail_run(run_id, "uocr-ffi runtime is missing: " + ffi_path().string());
    return;
  }

  UnlimitedOcrFfiEngine engine({ffi_path(), model_path(), mmproj_path()}, *profile);
  bool any_failed = false;
  {
    std::scoped_lock lock(mutex);
    runs[run_id].status = "running";
  }

  for (const auto& file : files) {
    const auto hash = stable_hash(file);
    {
      std::scoped_lock lock(mutex);
      auto& run = runs[run_id];
      if (run.cancel_requested) {
        run.status = "cancelled";
        return;
      }
      documents[hash].status = "running";
    }

    std::string error;
    OcrResult result;
    if (!is_image_file(file.absolute_path)) {
      error = "PDF rendering is not implemented in this C++ portable build yet";
    } else {
      result = engine.recognize_image({file.absolute_path, "document parsing.", profile->default_max_tokens},
                                      [](const OcrEvent&) {});
      if (!result.ok) {
        error = result.error.empty() ? "OCR failed" : result.error;
      }
    }

    std::scoped_lock lock(mutex);
    auto& document = documents[hash];
    if (!error.empty()) {
      any_failed = true;
      document.status = "failed";
      document.error = error;
    } else {
      const auto parsed = parse_ocr_markers(result.text, {.file_hash = hash, .page_no = 1});
      document.raw_text = result.text;
      document.cleaned_text = parsed.cleaned_text.empty() ? result.text : parsed.cleaned_text;
      document.boxes = to_overlay_boxes(parsed, 1);
      document.spans = parsed.text_region_spans;
      document.status = "completed";
    }
    runs[run_id].processed_pages += 1;
  }

  std::scoped_lock lock(mutex);
  auto& run = runs[run_id];
  run.status = any_failed ? "completed_with_errors" : "completed";
}

}  // namespace uocr::server
