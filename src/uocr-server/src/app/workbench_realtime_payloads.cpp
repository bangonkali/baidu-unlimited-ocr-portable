#include "workbench_state.hpp"

#include "uocr/app/realtime_event_hub.hpp"
#include "uocr/core/profiles.hpp"

namespace uocr::server {

void WorkbenchService::Impl::publish_event(std::string_view type, const Json::Value& payload) const {
  RealtimeEventHub::instance().publish(type, payload);
}

void WorkbenchService::Impl::publish_status_changed() const {
  Json::Value payload;
  {
    std::scoped_lock lock(mutex);
    payload = status_record();
  }
  publish_event("status.changed", payload);
}

Json::Value WorkbenchService::Impl::status_record() const {
  Json::Value payload;
  payload["state"] = "idle";
  payload["active_run_id"] = Json::nullValue;
  for (auto it = runs.rbegin(); it != runs.rend(); ++it) {
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
  payload["log_path"] = (app_root / "logs" / "uocr-server.log").string();
  payload["default_profile"] = default_ocr_profile().key;
  for (const auto* suffix : {".pdf", ".png", ".jpg", ".jpeg", ".bmp", ".tif", ".tiff", ".webp"}) {
    payload["supported_inputs"].append(suffix);
  }
  payload["realtime_path"] = "/api/events";
  return payload;
}

Json::Value WorkbenchService::Impl::document_page_record(const DocumentState& document,
                                                         const PageState& page) const {
  Json::Value payload;
  payload["file_hash"] = document.file_hash;
  payload["page_no"] = page.page_no;
  payload["status"] = page.status;
  payload["error"] = page.error.empty() ? Json::Value(Json::nullValue) : Json::Value(page.error);
  payload["width_px"] = page.width_px;
  payload["height_px"] = page.height_px;
  payload["dpi"] = page.dpi;
  payload["preview_available"] = !page.image_path.empty();
  payload["text_available"] = !page.cleaned_text.empty();
  payload["region_count"] = static_cast<Json::UInt64>(page.boxes.size());
  return payload;
}

Json::Value WorkbenchService::Impl::document_regions_record(const DocumentState& document) const {
  Json::Value payload;
  payload["file_hash"] = document.file_hash;
  payload["boxes"] = Json::arrayValue;
  for (const auto& page : document.pages) {
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

Json::Value WorkbenchService::Impl::document_text_record(const DocumentState& document) const {
  Json::Value payload;
  payload["file_hash"] = document.file_hash;
  payload["pages"] = Json::arrayValue;
  for (const auto& page_state : document.pages) {
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

}  // namespace uocr::server
