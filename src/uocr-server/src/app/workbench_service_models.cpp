#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include "uocr/core/profiles.hpp"

namespace uocr::server {

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

Json::Value WorkbenchService::start_model_download(const std::string& model_id, bool force) {
  if (model_id != kModelId) {
    return error_json("unknown model id");
  }
  impl_->start_download(force);
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = impl_->model_ready() ? "downloaded" : "downloading";
  return payload;
}

Json::Value WorkbenchService::cancel_model_download(const std::string& model_id) {
  if (model_id != kModelId) {
    return error_json("unknown model id");
  }
  impl_->cancel_download();
  Json::Value payload;
  payload["model_id"] = model_id;
  payload["status"] = impl_->model_downloading() ? "cancelling" : "idle";
  return payload;
}

Json::Value WorkbenchService::model_download_event(const std::string& model_id) const {
  if (model_id != kModelId) {
    return error_json("unknown model id");
  }
  std::scoped_lock lock(impl_->mutex);
  return impl_->model_event();
}

bool WorkbenchService::model_downloading(const std::string& model_id) const {
  if (model_id != kModelId) {
    return false;
  }
  std::scoped_lock lock(impl_->mutex);
  return impl_->model_downloading();
}

}  // namespace uocr::server
