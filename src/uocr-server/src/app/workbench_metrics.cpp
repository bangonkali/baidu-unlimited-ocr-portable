#include "workbench_state.hpp"

#include <algorithm>
#include <limits>
#include <map>
#include <string>
#include <utility>
#include <vector>

namespace uocr::server {
namespace {

struct Rollup {
  std::string id;
  std::string kind;
  std::string label;
  std::string run_id;
  std::string file_hash;
  int page_no = 0;
  std::string model_id;
  std::string runtime_id;
  std::string runtime_platform;
  std::string accelerator;
  std::string status = "completed";
  std::string error;
  std::uint64_t token_count = 0;
  std::uint64_t chunk_count = 0;
  std::uint64_t generation_duration_ms = 0;
  std::uint64_t elapsed_ms = 0;
  std::uint64_t first_token_latency_ms = 0;
  double min_tps = std::numeric_limits<double>::max();
  double max_tps = 0.0;
  std::size_t page_count = 0;
  std::string started_at;
  std::string completed_at;
  std::map<std::string, Rollup> children;
};

bool terminal_error_status(std::string_view status) {
  return status == "failed" || status == "cancelled";
}

std::string combined_status(std::string_view current, std::string_view next) {
  if (current == "running" || next == "running") {
    return "running";
  }
  if (current == "failed" || next == "failed") {
    return "failed";
  }
  if (current == "cancelled" || next == "cancelled") {
    return "cancelled";
  }
  if (current == "completed_with_errors" || next == "completed_with_errors") {
    return "completed_with_errors";
  }
  return "completed";
}

void absorb_metric(Rollup& rollup, const storage::OcrPageMetrics& row) {
  rollup.run_id = row.run_id;
  if (rollup.model_id.empty()) {
    rollup.model_id = row.model_id;
  }
  if (rollup.runtime_id.empty()) {
    rollup.runtime_id = row.runtime_id;
  }
  if (rollup.runtime_platform.empty()) {
    rollup.runtime_platform = row.runtime_platform;
  }
  if (rollup.accelerator.empty()) {
    rollup.accelerator = row.accelerator;
  }
  rollup.status = combined_status(rollup.status, row.status);
  if (rollup.error.empty() && terminal_error_status(row.status)) {
    rollup.error = row.error;
  }
  rollup.token_count += row.token_count;
  rollup.chunk_count += row.chunk_count;
  rollup.generation_duration_ms += row.generation_duration_ms;
  rollup.elapsed_ms += row.elapsed_ms;
  if (row.first_token_latency_ms > 0 &&
      (rollup.first_token_latency_ms == 0 || row.first_token_latency_ms < rollup.first_token_latency_ms)) {
    rollup.first_token_latency_ms = row.first_token_latency_ms;
  }
  if (row.min_tps > 0) {
    rollup.min_tps = std::min(rollup.min_tps, row.min_tps);
  }
  rollup.max_tps = std::max(rollup.max_tps, row.max_tps);
  rollup.page_count += 1;
  if (rollup.started_at.empty() || (!row.started_at.empty() && row.started_at < rollup.started_at)) {
    rollup.started_at = row.started_at;
  }
  if (rollup.completed_at.empty() || row.completed_at > rollup.completed_at) {
    rollup.completed_at = row.completed_at;
  }
}

double average_tps(const Rollup& rollup) {
  if (rollup.token_count == 0 || rollup.generation_duration_ms == 0) {
    return 0.0;
  }
  return static_cast<double>(rollup.token_count) /
         (static_cast<double>(rollup.generation_duration_ms) / 1000.0);
}

Json::Value metric_node(const Rollup& rollup) {
  Json::Value node;
  node["id"] = rollup.id;
  node["kind"] = rollup.kind;
  node["label"] = rollup.label;
  node["run_id"] = rollup.run_id;
  if (!rollup.file_hash.empty()) {
    node["file_hash"] = rollup.file_hash;
  }
  if (rollup.page_no > 0) {
    node["page_no"] = rollup.page_no;
  }
  node["status"] = rollup.status;
  node["model_id"] = rollup.model_id;
  node["runtime_id"] = rollup.runtime_id;
  node["runtime_platform"] = rollup.runtime_platform;
  node["accelerator"] = rollup.accelerator;
  node["token_count"] = static_cast<Json::UInt64>(rollup.token_count);
  node["chunk_count"] = static_cast<Json::UInt64>(rollup.chunk_count);
  node["page_count"] = static_cast<Json::UInt64>(rollup.page_count);
  node["first_token_latency_ms"] = static_cast<Json::UInt64>(rollup.first_token_latency_ms);
  node["generation_duration_ms"] = static_cast<Json::UInt64>(rollup.generation_duration_ms);
  node["elapsed_ms"] = static_cast<Json::UInt64>(rollup.elapsed_ms);
  node["min_tps"] = rollup.min_tps == std::numeric_limits<double>::max() ? 0.0 : rollup.min_tps;
  node["max_tps"] = rollup.max_tps;
  node["avg_tps"] = average_tps(rollup);
  node["started_at"] = rollup.started_at;
  node["completed_at"] = rollup.completed_at;
  node["error"] = rollup.error.empty() ? Json::Value(Json::nullValue) : Json::Value(rollup.error);
  node["children"] = Json::arrayValue;
  for (const auto& [_, child] : rollup.children) {
    node["children"].append(metric_node(child));
  }
  return node;
}

Rollup page_rollup(const storage::OcrPageMetrics& row) {
  Rollup page;
  page.id = "page:" + row.run_id + ":" + row.file_hash + ":" + std::to_string(row.page_no);
  page.kind = "page";
  page.label = "Page " + std::to_string(row.page_no);
  page.run_id = row.run_id;
  page.file_hash = row.file_hash;
  page.page_no = row.page_no;
  page.model_id = row.model_id;
  page.runtime_id = row.runtime_id;
  page.runtime_platform = row.runtime_platform;
  page.accelerator = row.accelerator;
  page.status = row.status;
  page.error = row.error;
  page.token_count = row.token_count;
  page.chunk_count = row.chunk_count;
  page.generation_duration_ms = row.generation_duration_ms;
  page.elapsed_ms = row.elapsed_ms;
  page.first_token_latency_ms = row.first_token_latency_ms;
  page.min_tps = row.min_tps > 0 ? row.min_tps : std::numeric_limits<double>::max();
  page.max_tps = row.max_tps;
  page.page_count = 1;
  page.started_at = row.started_at;
  page.completed_at = row.completed_at;
  return page;
}

}  // namespace

Json::Value WorkbenchService::Impl::metrics_tree_record(const std::vector<storage::OcrPageMetrics>& rows) const {
  std::map<std::string, Rollup> runs_by_id;
  for (const auto& row : rows) {
    auto& run = runs_by_id[row.run_id];
    if (run.id.empty()) {
      run.id = "run:" + row.run_id;
      run.kind = "run";
      run.label = row.run_id;
      run.run_id = row.run_id;
    }
    absorb_metric(run, row);

    auto& file = run.children[row.file_hash];
    if (file.id.empty()) {
      file.id = "file:" + row.run_id + ":" + row.file_hash;
      file.kind = "file";
      file.label = row.relative_path.empty() ? row.file_hash : row.relative_path;
      file.run_id = row.run_id;
      file.file_hash = row.file_hash;
    }
    absorb_metric(file, row);
    file.children[std::to_string(row.page_no)] = page_rollup(row);
  }

  Json::Value payload;
  payload["nodes"] = Json::arrayValue;
  for (const auto& [_, run] : runs_by_id) {
    payload["nodes"].append(metric_node(run));
  }
  return payload;
}

}  // namespace uocr::server
