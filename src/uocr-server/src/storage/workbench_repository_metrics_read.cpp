#include "workbench_repository_impl.hpp"

#include <algorithm>
#include <string>
#include <string_view>
#include <vector>

namespace uocr::storage {
namespace {

std::string unquote_json_string(std::string_view text) {
  if (text.size() < 2 || text.front() != '"' || text.back() != '"') {
    return std::string(text);
  }
  std::string value;
  value.reserve(text.size() - 2);
  for (std::size_t index = 1; index + 1 < text.size(); ++index) {
    const char ch = text[index];
    if (ch != '\\' || index + 2 >= text.size()) {
      value.push_back(ch);
      continue;
    }
    const char escaped = text[++index];
    switch (escaped) {
      case 'n':
        value.push_back('\n');
        break;
      case 'r':
        value.push_back('\r');
        break;
      case 't':
        value.push_back('\t');
        break;
      default:
        value.push_back(escaped);
        break;
    }
  }
  return value;
}

}  // namespace

std::vector<OcrPageMetrics> WorkbenchRepository::list_page_metrics(std::string_view run_id,
                                                                   std::size_t limit) const {
  std::scoped_lock lock(impl_->mutex);
  auto statement = impl_->statement(
      "SELECT m.run_id, m.file_hash, coalesce(l.relative_path, f.display_name, m.file_hash), m.page_no, "
      "m.engine_id, m.profile_id, m.model_id, coalesce(m.runtime_id, ''), "
      "coalesce(m.runtime_platform, ''), coalesce(m.accelerator, ''), m.status, coalesce(m.error, ''), "
      "m.token_count, m.chunk_count, m.first_token_latency_ms, m.generation_duration_ms, m.elapsed_ms, "
      "m.min_tps, m.max_tps, m.avg_tps, m.started_at, coalesce(m.first_token_at, ''), "
      "coalesce(m.completed_at, '') "
      "FROM ocr_page_metrics m "
      "LEFT JOIN files f ON f.file_hash = m.file_hash "
      "LEFT JOIN ("
      "  SELECT file_hash, relative_path FROM ("
      "    SELECT file_hash, relative_path, row_number() OVER (PARTITION BY file_hash ORDER BY observed_at DESC) AS rn "
      "    FROM file_locations"
      "  ) WHERE rn = 1"
      ") l ON l.file_hash = m.file_hash "
      "WHERE (? = '' OR m.run_id = ?) "
      "ORDER BY m.started_at DESC, m.run_id DESC, m.file_hash, m.page_no LIMIT ?");
  const std::string run_id_text(run_id);
  statement.bind_text(1, run_id_text);
  statement.bind_text(2, run_id_text);
  statement.bind_int32(3, static_cast<int>(std::max<std::size_t>(1, limit)));
  auto result = statement.query();

  std::vector<OcrPageMetrics> rows;
  for (idx_t row = 0; row < result.rows(); ++row) {
    OcrPageMetrics metrics;
    metrics.run_id = result.text(0, row);
    metrics.file_hash = result.text(1, row);
    metrics.relative_path = result.text(2, row);
    metrics.page_no = result.int32(3, row);
    metrics.engine_id = result.text(4, row);
    metrics.profile_id = result.text(5, row);
    metrics.model_id = result.text(6, row);
    metrics.runtime_id = result.text(7, row);
    metrics.runtime_platform = result.text(8, row);
    metrics.accelerator = result.text(9, row);
    metrics.status = result.text(10, row);
    metrics.error = result.text(11, row);
    metrics.token_count = result.uint64(12, row);
    metrics.chunk_count = result.uint64(13, row);
    metrics.first_token_latency_ms = result.uint64(14, row);
    metrics.generation_duration_ms = result.uint64(15, row);
    metrics.elapsed_ms = result.uint64(16, row);
    metrics.min_tps = result.number(17, row);
    metrics.max_tps = result.number(18, row);
    metrics.avg_tps = result.number(19, row);
    metrics.started_at = result.text(20, row);
    metrics.first_token_at = result.text(21, row);
    metrics.completed_at = result.text(22, row);
    rows.push_back(std::move(metrics));
  }
  return rows;
}

std::string WorkbenchRepository::setting_string(std::string_view key, std::string_view fallback) const {
  std::scoped_lock lock(impl_->mutex);
  auto statement = impl_->statement("SELECT coalesce(value::VARCHAR, '') FROM settings WHERE key = ?");
  statement.bind_text(1, key);
  auto result = statement.query();
  if (result.rows() == 0) {
    return std::string(fallback);
  }
  const auto value = unquote_json_string(result.text(0, 0));
  return value.empty() ? std::string(fallback) : value;
}

std::string WorkbenchRepository::setting_json(std::string_view key, std::string_view fallback_json) const {
  std::scoped_lock lock(impl_->mutex);
  auto statement = impl_->statement("SELECT coalesce(value::VARCHAR, '') FROM settings WHERE key = ?");
  statement.bind_text(1, key);
  auto result = statement.query();
  if (result.rows() == 0) {
    return std::string(fallback_json);
  }
  const auto value = result.text(0, 0);
  return value.empty() ? std::string(fallback_json) : value;
}

}  // namespace uocr::storage
