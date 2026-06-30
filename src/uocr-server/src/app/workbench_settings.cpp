#include "uocr/app/workbench_service.hpp"

#include "workbench_state.hpp"

#include <string>

#include "uocr/core/profiles.hpp"

namespace uocr::server {

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
  payload["workbench_ui"] = impl_->workbench_ui_settings();
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
    if (request.isMember("workbench_ui")) {
      std::string error;
      if (!impl_->update_workbench_ui_settings(request["workbench_ui"], error)) {
        return error_json(error);
      }
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

}  // namespace uocr::server
