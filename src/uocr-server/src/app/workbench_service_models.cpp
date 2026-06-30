#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include "uocr/app/app_logger.hpp"
#include "uocr/core/profiles.hpp"

namespace uocr::server {

Json::Value WorkbenchService::models() const {
  std::scoped_lock lock(impl_->mutex);
  Json::Value payload;
  payload["provider_repo"] = std::string(provider_repo_id());
  payload["provider_label"] = std::string(provider_label());
  payload["selected_model_id"] = impl_->selected_model_id;
  payload["shared_mmproj_file"] = std::string(shared_mmproj_file());
  payload["models"] = Json::arrayValue;
  for (const auto& entry : unlimited_ocr_model_catalog()) {
    payload["models"].append(impl_->model_record(entry));
  }
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

Json::Value WorkbenchService::select_model(const std::string& model_id) {
  const auto* entry = find_model_catalog_entry(model_id);
  if (entry == nullptr) {
    return error_json("unknown model id");
  }
  Json::Value event;
  {
    std::scoped_lock lock(impl_->mutex);
    impl_->selected_model_id = model_id;
    impl_->persist_selected_model();
    event = impl_->model_record(*entry);
  }
  if (impl_->logger) {
    impl_->logger->info("models", "selected model " + model_id);
  }
  impl_->publish_event("model.changed", event);
  impl_->publish_status_changed();
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = event["status"];
  return payload;
}

Json::Value WorkbenchService::start_model_download(const std::string& model_id, bool force) {
  if (find_model_catalog_entry(model_id) == nullptr) {
    return error_json("unknown model id");
  }
  impl_->start_download(model_id, force);
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = impl_->model_ready(model_id) ? "downloaded" : "downloading";
  return payload;
}

Json::Value WorkbenchService::cancel_model_download(const std::string& model_id) {
  if (find_model_catalog_entry(model_id) == nullptr) {
    return error_json("unknown model id");
  }
  impl_->cancel_download(model_id);
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = impl_->model_downloading(model_id) ? "cancelling" : "idle";
  return payload;
}

Json::Value WorkbenchService::model_download_event(const std::string& model_id) const {
  if (find_model_catalog_entry(model_id) == nullptr) {
    return error_json("unknown model id");
  }
  std::scoped_lock lock(impl_->mutex);
  return impl_->model_event(model_id);
}

bool WorkbenchService::model_downloading(const std::string& model_id) const {
  if (find_model_catalog_entry(model_id) == nullptr) {
    return false;
  }
  std::scoped_lock lock(impl_->mutex);
  return impl_->model_downloading(model_id);
}

}  // namespace uocr::server
