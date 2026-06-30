#include "workbench_state.hpp"

#include <algorithm>

namespace uocr::server {
namespace {

Json::Value model_file_json(const WorkbenchService::Impl::ModelState::File& file, bool downloading) {
  auto record = file;
  std::error_code error;
  if (!downloading && std::filesystem::exists(record.local_path, error)) {
    const auto size = static_cast<std::uint64_t>(std::filesystem::file_size(record.local_path, error));
    if (!error && size > 0) {
      record.status = "downloaded";
      record.downloaded_bytes = size;
      record.total_bytes = size;
      record.percent = 100.0;
    }
  }

  Json::Value value;
  value["file_id"] = record.file_id;
  value["file_name"] = record.file_name;
  value["status"] = record.status;
  value["local_path"] = record.local_path.string();
  value["downloaded_bytes"] = static_cast<Json::UInt64>(record.downloaded_bytes);
  value["total_bytes"] = record.total_bytes == 0 ? Json::Value(Json::nullValue)
                                                  : Json::Value(static_cast<Json::UInt64>(record.total_bytes));
  value["percent"] = record.percent;
  value["bytes_per_second"] = record.bytes_per_second;
  value["eta_seconds"] = record.eta_seconds < 0.0 ? Json::Value(Json::nullValue) : Json::Value(record.eta_seconds);
  value["error"] = record.error.empty() ? Json::Value(Json::nullValue) : Json::Value(record.error);
  return value;
}

}  // namespace

Json::Value WorkbenchService::Impl::model_record(const ModelCatalogEntry& entry) const {
  const auto state_found = models.find(std::string(entry.model_id));
  const ModelState empty_state;
  const auto& model = state_found == models.end() ? empty_state : state_found->second;
  Json::Value item;
  item["model_id"] = std::string(entry.model_id);
  item["display_name"] = std::string(entry.display_name);
  item["repo_id"] = std::string(provider_repo_id());
  item["revision"] = std::string(provider_revision());
  item["local_path"] = (app_root / "models").string();
  item["model_file"] = std::string(entry.model_file);
  item["mmproj_file"] = std::string(shared_mmproj_file());
  item["quantization"] = std::string(entry.quantization);
  item["bits"] = entry.bits;
  item["quality"] = std::string(entry.quality);
  item["hardware_tier"] = std::string(entry.hardware_tier);
  item["notes"] = std::string(entry.notes);
  item["recommended"] = entry.recommended;
  item["selected"] = entry.model_id == selected_model_id;
  item["provider_name"] = std::string(provider_label());
  item["total_required_bytes"] = static_cast<Json::UInt64>(entry.model_size_bytes + shared_mmproj_size_bytes());

  const auto ready = model_ready(entry.model_id);
  if (model.downloading) {
    item["status"] = "downloading";
  } else if (ready) {
    item["status"] = "downloaded";
  } else if (model.status == "cancelled" || model.status == "error") {
    item["status"] = model.status;
  } else {
    item["status"] = "missing";
  }

  if (item["status"].asString() == "error" && !model.error.empty()) {
    item["error"] = model.error;
  }
  item["current_file"] = model.current_file.empty() ? Json::Value(Json::nullValue) : Json::Value(model.current_file);
  item["status_message"] =
      model.status_message.empty() ? Json::Value(Json::nullValue) : Json::Value(model.status_message);
  item["downloaded_bytes"] = static_cast<Json::UInt64>(model.downloaded_bytes);
  item["total_bytes"] = model.total_bytes == 0 ? Json::Value(Json::nullValue)
                                               : Json::Value(static_cast<Json::UInt64>(model.total_bytes));
  item["overall_downloaded_bytes"] = static_cast<Json::UInt64>(model.downloaded_bytes);
  item["overall_total_bytes"] = model.total_bytes == 0 ? Json::Value(Json::nullValue)
                                                       : Json::Value(static_cast<Json::UInt64>(model.total_bytes));
  item["overall_percent"] = ready && !model.downloading ? 100.0 : model.overall_percent;
  item["bytes_per_second"] = model.bytes_per_second;
  item["eta_seconds"] = model.eta_seconds < 0.0 ? Json::Value(Json::nullValue) : Json::Value(model.eta_seconds);
  item["auth_available"] = model.auth_available;
  item["auth_source"] = model.auth_source.empty() ? Json::Value(Json::nullValue) : Json::Value(model.auth_source);
  item["last_event_at"] = model.last_event_at.empty() ? Json::Value(Json::nullValue) : Json::Value(model.last_event_at);
  item["files"] = Json::arrayValue;

  const auto files = model.files.empty() ? model_files(entry) : model.files;
  int downloaded_file_count = 0;
  for (const auto& file : files) {
    const auto file_json = model_file_json(file, model.downloading);
    if (file_json["status"].asString() == "downloaded") {
      ++downloaded_file_count;
    }
    item["files"].append(file_json);
  }
  item["downloaded_file_count"] = downloaded_file_count;
  item["total_file_count"] = static_cast<int>(files.size());

  std::uintmax_t size = 0;
  std::error_code error;
  const auto path = model_path(entry.model_id);
  if (!path.empty() && std::filesystem::exists(path, error)) {
    size += std::filesystem::file_size(path, error);
  }
  if (std::filesystem::exists(mmproj_path())) {
    size += std::filesystem::file_size(mmproj_path(), error);
  }
  item["size_bytes"] = static_cast<Json::UInt64>(size);
  return item;
}

Json::Value WorkbenchService::Impl::model_event(std::string_view model_id) const {
  const auto* entry = find_model_catalog_entry(model_id);
  if (entry == nullptr) {
    return error_json("unknown model id");
  }
  const auto state_found = models.find(std::string(model_id));
  const ModelState empty_state;
  const auto& model = state_found == models.end() ? empty_state : state_found->second;
  Json::Value event = model_record(*entry);
  event["phase"] = model.status;
  event["message"] = model.status_message.empty() ? event["status"] : Json::Value(model.status_message);
  return event;
}

bool WorkbenchService::Impl::model_downloading(std::string_view model_id) const {
  const auto found = models.find(std::string(model_id));
  return found != models.end() && found->second.downloading;
}

bool WorkbenchService::Impl::any_model_downloading() const {
  return std::any_of(models.begin(), models.end(), [](const auto& item) {
    return item.second.downloading;
  });
}

}  // namespace uocr::server
