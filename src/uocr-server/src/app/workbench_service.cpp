#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include <utility>

#include "uocr/app/app_logger.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

WorkbenchService::WorkbenchService(std::filesystem::path app_root, std::shared_ptr<AppLogger> logger)
    : impl_(std::make_shared<Impl>(std::move(app_root), std::move(logger))) {}

Json::Value WorkbenchService::status() const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["state"] = "idle";
  payload["active_run_id"] = Json::nullValue;
  for (auto it = impl_->runs.rbegin(); it != impl_->runs.rend(); ++it) {
    if (it->second.status == "queued" || it->second.status == "running") {
      payload["state"] = it->second.status;
      payload["active_run_id"] = it->second.run_id;
      break;
    }
  }
  payload["host"] = "127.0.0.1";
  payload["runtime_platform"] =
#ifdef _WIN32
      "windows-x86_64-cuda13";
#else
      "linux-x86_64-cuda13";
#endif
  payload["accelerator"] = "cuda";
  payload["inference_engine"] = "Unlimited-OCR FFI";
  payload["log_path"] = (impl_->app_root / "logs" / "uocr-server.log").string();
  payload["default_profile"] = default_ocr_profile().key;
  for (const auto* suffix : {".pdf", ".png", ".jpg", ".jpeg", ".bmp", ".tif", ".tiff", ".webp"}) {
    payload["supported_inputs"].append(suffix);
  }
  return payload;
}

Json::Value WorkbenchService::settings() const {
  Json::Value payload;
  payload["pdf_dpi"] = 200;
  payload["ocr_concurrency"] = 1;
  payload["default_profile"] = default_ocr_profile().key;
  payload["retry_profile"] = "experimental-exact-prefill-q4";
  payload["cache_path"] = (impl_->app_root / "cache").string();
  payload["database_path"] = (impl_->app_root / "data" / "uocr.duckdb").string();
  return payload;
}

Json::Value WorkbenchService::start_ingest(const Json::Value& request) {
  const std::string root = request.get("root_path", "").asString();
  const std::string profile = request.get("profile_id", default_ocr_profile().key).asString();
  const auto files = discover_supported_files(root);
  const auto run_id = now_id();
  if (impl_->logger) {
    impl_->logger->info("ingest", "scan requested for " + root + " found " + std::to_string(files.size()) +
                                      " supported files");
  }
  {
    std::scoped_lock lock(impl_->mutex);
    Impl::RunState run;
    run.run_id = run_id;
    run.root_path = root;
    run.queued_files = static_cast<int>(files.size());
    run.total_pages = static_cast<int>(files.size());
    for (const auto& file : files) {
      Impl::DocumentState document;
      document.file_hash = stable_hash(file);
      document.absolute_path = file.absolute_path;
      document.relative_path = file.relative_path;
      run.file_hashes.push_back(document.file_hash);
      impl_->documents[document.file_hash] = std::move(document);
    }
    impl_->runs[run_id] = run;
  }
  impl_->start_run(run_id, files, profile);
  return get_run(run_id);
}

Json::Value WorkbenchService::list_runs() const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["runs"] = Json::arrayValue;
  for (auto it = impl_->runs.rbegin(); it != impl_->runs.rend(); ++it) {
    payload["runs"].append(impl_->run_record(it->second));
  }
  return payload;
}

Json::Value WorkbenchService::get_run(const std::string& run_id) const {
  std::scoped_lock lock(impl_->mutex);
  const auto found = impl_->runs.find(run_id);
  return found == impl_->runs.end() ? error_json("run not found") : impl_->run_record(found->second);
}

Json::Value WorkbenchService::run_command(const std::string& run_id, const std::string& command) {
  std::scoped_lock lock(impl_->mutex);
  auto found = impl_->runs.find(run_id);
  if (found == impl_->runs.end()) {
    return error_json("run not found");
  }
  if (command != "stop") {
    return error_json("unsupported run command");
  }
  found->second.cancel_requested = true;
  found->second.status = "cancelled";
  for (const auto& hash : found->second.file_hashes) {
    auto& document = impl_->documents[hash];
    if (document.status == "queued" || document.status == "running" || document.status == "rendering") {
      document.status = "cancelled";
    }
  }
  if (impl_->logger) {
    impl_->logger->warn("ingest", "stop requested for run " + run_id);
  }
  return impl_->run_record(found->second);
}

std::string WorkbenchService::run_event_stream(const std::string& run_id) const {
  Json::StreamWriterBuilder builder;
  builder["indentation"] = "";
  return "event: snapshot\ndata: " + Json::writeString(builder, get_run(run_id)) + "\n\n";
}

Json::Value WorkbenchService::list_documents(const std::string& query) const {
  std::scoped_lock lock(impl_->mutex);
  const auto needle = lower(query);
  Json::Value payload;
  payload["documents"] = Json::arrayValue;
  for (const auto& [_, document] : impl_->documents) {
    const auto haystack = lower(document.relative_path.generic_string() + " " + document.cleaned_text);
    if (!needle.empty() && haystack.find(needle) == std::string::npos) {
      continue;
    }
    payload["documents"].append(impl_->document_summary(document));
  }
  return payload;
}

Json::Value WorkbenchService::get_document(const std::string& file_hash) const {
  std::scoped_lock lock(impl_->mutex);
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end()) {
    return error_json("document not found");
  }
  auto value = impl_->document_summary(found->second);
  value["absolute_path"] = found->second.absolute_path.string();
  return value;
}

Json::Value WorkbenchService::document_regions(const std::string& file_hash) const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["file_hash"] = file_hash;
  payload["boxes"] = Json::arrayValue;
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end()) {
    return payload;
  }
  for (const auto& page : found->second.pages) {
    for (const auto& box : page.boxes) {
      Json::Value item;
      item["region_id"] = box.region_id;
      item["label"] = box.label;
      item["page_no"] = box.page_no;
      item["left_percent"] = box.left_percent;
      item["top_percent"] = box.top_percent;
      item["width_percent"] = box.width_percent;
      item["height_percent"] = box.height_percent;
      item["hidden"] = box.hidden;
      payload["boxes"].append(item);
    }
  }
  return payload;
}

Json::Value WorkbenchService::document_text(const std::string& file_hash) const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["file_hash"] = file_hash;
  payload["pages"] = Json::arrayValue;
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end()) {
    return payload;
  }
  for (const auto& page_state : found->second.pages) {
    Json::Value page;
    page["page_no"] = page_state.page_no;
    page["text"] = page_state.cleaned_text;
    page["spans"] = Json::arrayValue;
    for (const auto& span : page_state.spans) {
      Json::Value item;
      item["region_id"] = span.region_id;
      item["page_no"] = span.page_no;
      item["start"] = static_cast<Json::UInt64>(span.start);
      item["end"] = static_cast<Json::UInt64>(span.end);
      page["spans"].append(item);
    }
    payload["pages"].append(page);
  }
  return payload;
}

Json::Value WorkbenchService::document_preview_images(const std::string& file_hash) const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["file_hash"] = file_hash;
  payload["variants"] = Json::arrayValue;
  payload["pages"] = Json::arrayValue;
  const auto found = impl_->documents.find(file_hash);
  if (found != impl_->documents.end() && (impl_->is_image_document(found->second) || !found->second.pages.empty())) {
    payload["variants"].append("source");
    const auto page_count = found->second.pages.empty() ? 1 : static_cast<int>(found->second.pages.size());
    for (int page = 1; page <= page_count; ++page) {
      payload["pages"].append(page);
    }
  }
  return payload;
}

std::optional<std::filesystem::path> WorkbenchService::document_preview_image(const std::string& file_hash,
                                                                              const std::string& variant,
                                                                              int page_no) const {
  std::scoped_lock lock(impl_->mutex);
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end() || variant != "source") {
    return std::nullopt;
  }
  if (impl_->is_image_document(found->second) && page_no == 1) {
    return found->second.absolute_path;
  }
  for (const auto& page : found->second.pages) {
    if (page.page_no == page_no && !page.image_path.empty()) {
      return page.image_path;
    }
  }
  return std::nullopt;
}

}  // namespace uocr::server
