#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include <exception>
#include <stdexcept>
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
  std::scoped_lock lock(impl_->mutex);
  const auto runtime = impl_->selected_runtime();
  Json::Value payload;
  payload["pdf_dpi"] = 200;
  payload["ocr_concurrency"] = 1;
  payload["default_profile"] = impl_->selected_profile_id;
  payload["retry_profile"] = "best-zero-empty-q4";
  payload["cache_path"] = (impl_->app_root / "cache").string();
  payload["database_path"] = (impl_->app_root / "data" / "uocr.duckdb").string();
  payload["selected_runtime_id"] = runtime.runtime_id;
  payload["selected_accelerator"] = runtime.accelerator;
  payload["selected_model_id"] = impl_->selected_model_id;
  payload["runtime_variants"] = Json::arrayValue;
  for (const auto& variant : impl_->runtime_variants()) {
    Json::Value item;
    item["runtime_id"] = variant.runtime_id;
    item["label"] = variant.label;
    item["platform"] = variant.platform;
    item["accelerator"] = variant.accelerator;
    item["backend"] = variant.backend;
    item["ffi_library"] = variant.ffi_library.string();
    item["installed"] = variant.installed;
    item["hardware_supported"] = variant.hardware_supported;
    item["selectable"] = variant.selectable;
    item["selected"] = variant.runtime_id == runtime.runtime_id;
    item["support_detail"] = variant.support_detail;
    payload["runtime_variants"].append(item);
  }
  return payload;
}

Json::Value WorkbenchService::update_settings(const Json::Value& request) {
  {
    std::scoped_lock lock(impl_->mutex);
    const auto requested_runtime = request.get("selected_runtime_id", "").asString();
    if (!requested_runtime.empty() && !impl_->select_runtime(requested_runtime)) {
      return error_json("runtime is not supported on this device or is not installed: " + requested_runtime);
    }
    const auto requested_profile = request.get("default_profile", "").asString();
    if (!requested_profile.empty()) {
      if (find_ocr_profile(requested_profile) == nullptr) {
        return error_json("unknown OCR profile: " + requested_profile);
      }
      impl_->selected_profile_id = requested_profile;
      impl_->persist_selected_profile();
    }
  }
  impl_->publish_status_changed();
  return settings();
}

Json::Value WorkbenchService::start_ingest(const Json::Value& request) {
  const std::string root = request.get("root_path", "").asString();
  std::string default_profile;
  {
    std::scoped_lock lock(impl_->mutex);
    default_profile = impl_->selected_profile_id;
  }
  const std::string profile = request.get("profile_id", default_profile).asString();
  const std::string requested_model = request.get("model_id", "").asString();
  std::string model_id;
  {
    std::scoped_lock lock(impl_->mutex);
    model_id = requested_model.empty() ? impl_->selected_model_id : requested_model;
  }
  if (find_model_catalog_entry(model_id) == nullptr) {
    throw std::runtime_error("unknown model id: " + model_id);
  }
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
    run.profile_id = profile;
    run.model_id = model_id;
    run.runtime_id = impl_->selected_runtime().runtime_id;
    for (const auto& file : files) {
      Impl::DocumentState document;
      document.file_hash = stable_hash(file);
      document.absolute_path = file.absolute_path;
      document.relative_path = file.relative_path;
      run.file_hashes.push_back(document.file_hash);
      impl_->documents[document.file_hash] = std::move(document);
      impl_->persist_document(impl_->documents[run.file_hashes.back()], root);
      document_events.push_back(impl_->document_summary(impl_->documents[run.file_hashes.back()]));
    }
    impl_->runs[run_id] = run;
    impl_->persist_run(impl_->runs[run_id]);
    run_event = impl_->run_record(impl_->runs[run_id]);
  }
  impl_->publish_event("run.changed", run_event);
  for (const auto& document_event : document_events) {
    impl_->publish_event("document.changed", document_event);
  }
  impl_->publish_status_changed();
  impl_->start_run(run_id, files, profile, model_id);
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
      impl_->persist_document(document, found->second.root_path);
      document_events.push_back(impl_->document_summary(document));
    }
    impl_->persist_run(found->second);
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
  std::vector<std::string> persisted_hits;
  bool used_persisted_search = false;
  if (!query.empty() && impl_->repository) {
    try {
      persisted_hits = impl_->repository->search_document_hashes(query, 200);
      used_persisted_search = true;
    } catch (const std::exception& error) {
      if (impl_->logger) {
        impl_->logger->error("database", std::string("DuckDB search failed: ") + error.what());
      }
    }
  }

  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["documents"] = Json::arrayValue;
  if (used_persisted_search) {
    for (const auto& hash : persisted_hits) {
      const auto found = impl_->documents.find(hash);
      if (found != impl_->documents.end()) {
        payload["documents"].append(impl_->document_summary(found->second));
      }
    }
    return payload;
  }

  const auto needle = lower(query);
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
