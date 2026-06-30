#include "workbench_state.hpp"

#include <json/json.h>

#include <memory>
#include <string>
#include <string_view>

namespace uocr::server {
namespace {

constexpr std::string_view kWorkbenchUiSettingKey = "workbench_ui";
constexpr std::string_view kDefaultWorkbenchUiJson =
    R"({"theme":"dark","auto_follow_regions":true,"overlay_visible":true,"labels_visible":true,"panes_collapsed":{"explorer":false,"details":true,"diagnostics":true}})";

Json::Value default_workbench_ui_settings() {
  Json::Value panes;
  panes["explorer"] = false;
  panes["details"] = true;
  panes["diagnostics"] = true;

  Json::Value value;
  value["theme"] = "dark";
  value["auto_follow_regions"] = true;
  value["overlay_visible"] = true;
  value["labels_visible"] = true;
  value["panes_collapsed"] = panes;
  return value;
}

bool parse_json_object(std::string_view text, Json::Value& output) {
  Json::CharReaderBuilder builder;
  std::string error;
  const std::unique_ptr<Json::CharReader> reader(builder.newCharReader());
  const auto* begin = text.data();
  const auto* end = begin + text.size();
  if (!reader->parse(begin, end, &output, &error)) {
    return false;
  }
  return output.isObject();
}

bool merge_bool(const Json::Value& patch, Json::Value& target, const char* key, std::string& error) {
  if (!patch.isMember(key)) {
    return true;
  }
  if (!patch[key].isBool()) {
    error = std::string("workbench_ui.") + key + " must be a boolean";
    return false;
  }
  target[key] = patch[key].asBool();
  return true;
}

bool merge_pane(const Json::Value& patch, Json::Value& target, const char* pane, std::string& error) {
  if (!patch.isMember(pane)) {
    return true;
  }
  if (!patch[pane].isBool()) {
    error = std::string("workbench_ui.panes_collapsed.") + pane + " must be a boolean";
    return false;
  }
  target[pane] = patch[pane].asBool();
  return true;
}

bool merge_workbench_ui(const Json::Value& patch, Json::Value& target, std::string& error) {
  if (!patch.isObject()) {
    error = "workbench_ui must be an object";
    return false;
  }
  if (patch.isMember("theme")) {
    if (!patch["theme"].isString()) {
      error = "workbench_ui.theme must be a string";
      return false;
    }
    const auto theme = patch["theme"].asString();
    if (theme != "dark" && theme != "light") {
      error = "workbench_ui.theme must be dark or light";
      return false;
    }
    target["theme"] = theme;
  }
  if (!merge_bool(patch, target, "auto_follow_regions", error) ||
      !merge_bool(patch, target, "overlay_visible", error) ||
      !merge_bool(patch, target, "labels_visible", error)) {
    return false;
  }
  if (!patch.isMember("panes_collapsed")) {
    return true;
  }
  const auto& panes = patch["panes_collapsed"];
  if (!panes.isObject()) {
    error = "workbench_ui.panes_collapsed must be an object";
    return false;
  }
  auto target_panes = target["panes_collapsed"];
  if (!merge_pane(panes, target_panes, "explorer", error) ||
      !merge_pane(panes, target_panes, "details", error) ||
      !merge_pane(panes, target_panes, "diagnostics", error)) {
    return false;
  }
  target["panes_collapsed"] = target_panes;
  return true;
}

std::string compact_json(const Json::Value& value) {
  Json::StreamWriterBuilder builder;
  builder["indentation"] = "";
  return Json::writeString(builder, value);
}

}  // namespace

Json::Value WorkbenchService::Impl::workbench_ui_settings() const {
  auto value = default_workbench_ui_settings();
  if (!repository) {
    return value;
  }
  Json::Value persisted;
  const auto persisted_text = repository->setting_json(kWorkbenchUiSettingKey, kDefaultWorkbenchUiJson);
  std::string error;
  if (parse_json_object(persisted_text, persisted)) {
    static_cast<void>(merge_workbench_ui(persisted, value, error));
  }
  return value;
}

bool WorkbenchService::Impl::update_workbench_ui_settings(const Json::Value& patch, std::string& error) const {
  auto merged = workbench_ui_settings();
  if (!merge_workbench_ui(patch, merged, error)) {
    return false;
  }
  if (repository) {
    repository->put_setting_json(kWorkbenchUiSettingKey, compact_json(merged));
  }
  return true;
}

}  // namespace uocr::server
