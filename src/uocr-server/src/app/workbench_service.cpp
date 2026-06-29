#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include <utility>

#include "uocr/core/profiles.hpp"
#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

WorkbenchService::WorkbenchService(std::filesystem::path app_root)
    : impl_(std::make_shared<Impl>(std::move(app_root))) {}

Json::Value WorkbenchService::status() const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["state"] = "idle";
  payload["active_run_id"] = Json::nullValue;
  for (auto it = impl_->runs.rbegin(); it != impl_->runs.rend(); ++it) {
    if (it->second.status == "queued" || it->second.status == "running" || it->second.status == "paused") {
      payload["state"] = it->second.status;
      payload["active_run_id"] = it->second.run_id;
      break;
    }
  }
  payload["host"] = "127.0.0.1";
  payload["default_profile"] = default_ocr_profile().key;
  for (const auto* suffix : {".pdf", ".png", ".jpg", ".jpeg", ".bmp", ".tif", ".tiff", ".webp"}) {
    payload["supported_inputs"].append(suffix);
  }
  return payload;
}

Json::Value WorkbenchService::models() const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["models"].append(impl_->model_record());
  payload["profiles"] = Json::arrayValue;
  for (const auto& profile : ocr_profiles()) {
    Json::Value item;
    item["key"] = profile.key;
    item["label"] = profile.label;
    item["engine_name"] = profile.engine_name;
    item["description"] = profile.description;
    item["default_max_tokens"] = profile.default_max_tokens;
    item["ngram_size"] = profile.ngram_size;
    item["ngram_window"] = profile.ngram_window;
    item["pdf_ngram_window"] = profile.pdf_ngram_window;
    item["force_prompt_eos"] = profile.force_prompt_eos;
    item["no_image_end"] = profile.no_image_end;
    payload["profiles"].append(item);
  }
  return payload;
}

Json::Value WorkbenchService::start_model_download(const std::string& model_id) {
  if (model_id != "unlimited-ocr-q4-k-m") {
    return error_json("unknown model id");
  }
  impl_->start_download();
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = impl_->model_ready() ? "downloaded" : "downloading";
  return payload;
}

Json::Value WorkbenchService::cancel_model_download(const std::string& model_id) {
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = "cancel_not_supported";
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
  if (command == "pause") {
    found->second.cancel_requested = true;
    found->second.status = "paused";
  } else if (command == "stop") {
    found->second.cancel_requested = true;
    found->second.status = "cancelled";
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
  for (const auto& box : found->second.boxes) {
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
  Json::Value page;
  page["page_no"] = 1;
  page["text"] = found->second.cleaned_text;
  page["spans"] = Json::arrayValue;
  for (const auto& span : found->second.spans) {
    Json::Value item;
    item["region_id"] = span.region_id;
    item["page_no"] = span.page_no;
    item["start"] = static_cast<Json::UInt64>(span.start);
    item["end"] = static_cast<Json::UInt64>(span.end);
    page["spans"].append(item);
  }
  payload["pages"].append(page);
  return payload;
}

}  // namespace uocr::server
