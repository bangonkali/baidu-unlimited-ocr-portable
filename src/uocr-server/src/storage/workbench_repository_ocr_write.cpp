#include "workbench_repository_impl.hpp"

#include <cctype>
#include <map>
#include <string>
#include <tuple>
#include <vector>

namespace uocr::storage {
namespace {

double normalized_x1(const OverlayBox& box) {
  return box.left_percent / 100.0 * 999.0;
}

double normalized_y1(const OverlayBox& box) {
  return box.top_percent / 100.0 * 999.0;
}

double normalized_x2(const OverlayBox& box) {
  return (box.left_percent + box.width_percent) / 100.0 * 999.0;
}

double normalized_y2(const OverlayBox& box) {
  return (box.top_percent + box.height_percent) / 100.0 * 999.0;
}

std::string work_unit_id(const std::string& run_id, const std::string& file_hash, int page_no) {
  return run_id + ":" + file_hash + ":" + std::to_string(page_no);
}

std::map<std::string, TextRegionSpan> span_by_region(const StoredPage& page) {
  std::map<std::string, TextRegionSpan> spans;
  for (const auto& span : page.spans) {
    spans.try_emplace(span.region_id, span);
  }
  return spans;
}

std::string region_content(const StoredPage& page, const OverlayBox& box) {
  for (const auto& span : page.spans) {
    if (span.region_id == box.region_id && span.end <= page.cleaned_text.size() && span.start <= span.end) {
      return page.cleaned_text.substr(span.start, span.end - span.start);
    }
  }
  return box.content_markdown.empty() ? box.label : box.content_markdown;
}

std::vector<std::tuple<std::string, std::size_t, std::size_t>> terms_for(std::string_view text) {
  std::vector<std::tuple<std::string, std::size_t, std::size_t>> terms;
  std::size_t start = std::string_view::npos;
  std::string token;
  for (std::size_t index = 0; index <= text.size(); ++index) {
    const auto ch = index < text.size() ? static_cast<unsigned char>(text[index]) : 0;
    if (index < text.size() && std::isalnum(ch)) {
      if (start == std::string_view::npos) {
        start = index;
        token.clear();
      }
      token.push_back(static_cast<char>(std::tolower(ch)));
      continue;
    }
    if (start != std::string_view::npos) {
      terms.emplace_back(token, start, index);
      start = std::string_view::npos;
    }
  }
  return terms;
}

void bind_nullable_text(Statement& statement, idx_t index, std::string_view value) {
  if (value.empty()) {
    statement.bind_null(index);
  } else {
    statement.bind_text(index, value);
  }
}

void delete_page_ocr_rows(const WorkbenchRepository::Impl& impl,
                          const std::string& file_hash,
                          int page_no,
                          std::string_view engine_id,
                          std::string_view profile_id) {
  auto delete_ocr = impl.statement(
      "DELETE FROM document_page_ocr WHERE file_hash = ? AND page_no = ? AND engine_id = ? AND profile_id = ?");
  delete_ocr.bind_text(1, file_hash);
  delete_ocr.bind_int32(2, page_no);
  delete_ocr.bind_text(3, engine_id);
  delete_ocr.bind_text(4, profile_id);
  delete_ocr.execute();
}

}  // namespace

void WorkbenchRepository::replace_page_ocr(const std::string& file_hash,
                                           const StoredPage& page,
                                           std::string_view engine_id,
                                           std::string_view profile_id) {
  std::scoped_lock lock(impl_->mutex);
  auto delete_document = impl_->statement("DELETE FROM ocr_documents WHERE file_hash = ?");
  delete_document.bind_text(1, file_hash);
  delete_document.execute();

  auto document = impl_->statement(
      "INSERT INTO ocr_documents(file_hash, engine_id, profile_id, runtime_metadata, status, updated_at) "
      "VALUES (?, ?, ?, CAST(? AS JSON), ?, current_timestamp)");
  document.bind_text(1, file_hash);
  document.bind_text(2, engine_id);
  document.bind_text(3, profile_id);
  document.bind_text(4, "{}");
  document.bind_text(5, page.status);
  document.execute();

  delete_page_ocr_rows(*impl_, file_hash, page.page_no, engine_id, profile_id);
  auto ocr = impl_->statement(
      "INSERT INTO document_page_ocr(file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text, "
      "status, attempts, error, options) VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, CAST(? AS JSON))");
  ocr.bind_text(1, file_hash);
  ocr.bind_int32(2, page.page_no);
  ocr.bind_text(3, engine_id);
  ocr.bind_text(4, profile_id);
  ocr.bind_text(5, page.raw_text);
  ocr.bind_text(6, page.cleaned_text);
  ocr.bind_text(7, page.status);
  bind_nullable_text(ocr, 8, page.error);
  ocr.bind_text(9, "{}");
  ocr.execute();

  auto delete_links = impl_->statement("DELETE FROM document_text_region_links WHERE file_hash = ? AND page_no = ?");
  delete_links.bind_text(1, file_hash);
  delete_links.bind_int32(2, page.page_no);
  delete_links.execute();
  auto delete_terms = impl_->statement("DELETE FROM document_terms WHERE file_hash = ? AND page_no = ?");
  delete_terms.bind_text(1, file_hash);
  delete_terms.bind_int32(2, page.page_no);
  delete_terms.execute();
  auto delete_regions = impl_->statement(
      "DELETE FROM document_regions WHERE file_hash = ? AND page_no = ? AND engine_id = ? AND profile_id = ?");
  delete_regions.bind_text(1, file_hash);
  delete_regions.bind_int32(2, page.page_no);
  delete_regions.bind_text(3, engine_id);
  delete_regions.bind_text(4, profile_id);
  delete_regions.execute();

  const auto spans = span_by_region(page);
  for (const auto& box : page.boxes) {
    auto region = impl_->statement(
        "INSERT INTO document_regions(region_id, file_hash, page_no, engine_id, profile_id, label, x1, y1, x2, "
        "y2, source_span_start, source_span_end, content_markdown, content_html) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)");
    region.bind_text(1, box.region_id);
    region.bind_text(2, file_hash);
    region.bind_int32(3, page.page_no);
    region.bind_text(4, engine_id);
    region.bind_text(5, profile_id);
    region.bind_text(6, box.label);
    region.bind_double(7, normalized_x1(box));
    region.bind_double(8, normalized_y1(box));
    region.bind_double(9, normalized_x2(box));
    region.bind_double(10, normalized_y2(box));
    const auto found = spans.find(box.region_id);
    if (found == spans.end()) {
      region.bind_null(11);
      region.bind_null(12);
    } else {
      region.bind_uint64(11, found->second.start);
      region.bind_uint64(12, found->second.end);
    }
    region.bind_text(13, region_content(page, box));
    bind_nullable_text(region, 14, box.content_html);
    region.execute();
  }

  for (const auto& span : page.spans) {
    auto link = impl_->statement(
        "INSERT INTO document_text_region_links(file_hash, page_no, region_id, text_start, text_end) "
        "VALUES (?, ?, ?, ?, ?)");
    link.bind_text(1, file_hash);
    link.bind_int32(2, page.page_no);
    link.bind_text(3, span.region_id);
    link.bind_uint64(4, span.start);
    link.bind_uint64(5, span.end);
    link.execute();
  }

  for (const auto& [term, start, end] : terms_for(page.cleaned_text)) {
    auto insert = impl_->statement(
        "INSERT INTO document_terms(file_hash, page_no, term, text_start, text_end) VALUES (?, ?, ?, ?, ?)");
    insert.bind_text(1, file_hash);
    insert.bind_int32(2, page.page_no);
    insert.bind_text(3, term);
    insert.bind_uint64(4, start);
    insert.bind_uint64(5, end);
    insert.execute();
  }
}

void WorkbenchRepository::upsert_work_unit(const std::string& run_id,
                                           const std::string& file_hash,
                                           int page_no,
                                           std::string_view status,
                                           int attempts,
                                           std::string_view error) {
  std::scoped_lock lock(impl_->mutex);
  auto delete_existing = impl_->statement("DELETE FROM ingest_work_units WHERE work_unit_id = ?");
  delete_existing.bind_text(1, work_unit_id(run_id, file_hash, page_no));
  delete_existing.execute();

  auto statement = impl_->statement(
      "INSERT INTO ingest_work_units(work_unit_id, run_id, file_hash, page_no, status, attempts, error, "
      "started_at, finished_at) VALUES (?, ?, ?, ?, ?, ?, ?, "
      "CASE WHEN ? = 'running' THEN current_timestamp ELSE NULL END, "
      "CASE WHEN ? IN ('completed','failed','cancelled') THEN current_timestamp ELSE NULL END)");
  statement.bind_text(1, work_unit_id(run_id, file_hash, page_no));
  statement.bind_text(2, run_id);
  statement.bind_text(3, file_hash);
  statement.bind_int32(4, page_no);
  statement.bind_text(5, status);
  statement.bind_int32(6, attempts);
  bind_nullable_text(statement, 7, error);
  statement.bind_text(8, status);
  statement.bind_text(9, status);
  statement.execute();
}

void WorkbenchRepository::append_diagnostic_event(const std::string& run_id,
                                                  std::string_view level,
                                                  std::string_view message) {
  std::scoped_lock lock(impl_->mutex);
  auto statement = impl_->statement(
      "INSERT INTO ingest_diagnostic_events(event_id, run_id, level, message, attributes) "
      "VALUES (?, ?, ?, ?, CAST(? AS JSON))");
  statement.bind_text(1, make_event_id(run_id + std::string(message)));
  statement.bind_text(2, run_id);
  statement.bind_text(3, level);
  statement.bind_text(4, message);
  statement.bind_text(5, "{}");
  statement.execute();
}

}  // namespace uocr::storage
