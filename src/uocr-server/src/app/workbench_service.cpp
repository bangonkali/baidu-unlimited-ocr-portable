#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include <utility>
#include <vector>

#include "uocr/app/app_logger.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

WorkbenchService::WorkbenchService(std::filesystem::path app_root, std::shared_ptr<AppLogger> logger)
    : impl_(std::make_shared<Impl>(std::move(app_root), std::move(logger))) {}

Json::Value WorkbenchService::status() const {
  std::scoped_lock lock(impl_->mutex);
  return impl_->status_record();
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
  Json::Value run_event;
  std::vector<Json::Value> document_events;
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
      document_events.push_back(impl_->document_summary(impl_->documents[run.file_hashes.back()]));
    }
    impl_->runs[run_id] = run;
    run_event = impl_->run_record(impl_->runs[run_id]);
  }
  impl_->publish_event("run.changed", run_event);
  for (const auto& document_event : document_events) {
    impl_->publish_event("document.changed", document_event);
  }
  impl_->publish_status_changed();
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
  Json::Value run_event;
  std::vector<Json::Value> document_events;
  {
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
      document_events.push_back(impl_->document_summary(document));
    }
    run_event = impl_->run_record(found->second);
  }
  if (impl_->logger) {
    impl_->logger->warn("ingest", "stop requested for run " + run_id);
  }
  impl_->publish_event("run.changed", run_event);
  for (const auto& document_event : document_events) {
    impl_->publish_event("document.changed", document_event);
  }
  impl_->publish_status_changed();
  return run_event;
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
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end()) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["boxes"] = Json::arrayValue;
    return payload;
  }
  return impl_->document_regions_record(found->second);
}

Json::Value WorkbenchService::document_text(const std::string& file_hash) const {
  std::scoped_lock lock(impl_->mutex);
  const auto found = impl_->documents.find(file_hash);
  if (found == impl_->documents.end()) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["pages"] = Json::arrayValue;
    return payload;
  }
  return impl_->document_text_record(found->second);
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
