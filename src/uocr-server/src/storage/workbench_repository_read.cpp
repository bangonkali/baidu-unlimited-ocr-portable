#include "workbench_repository_impl.hpp"

#include <algorithm>
#include <cctype>
#include <map>
#include <set>
#include <vector>

namespace uocr::storage {
namespace {

OverlayBox overlay_from_row(const QueryResult& result, idx_t row) {
  const auto x1 = result.number(6, row);
  const auto y1 = result.number(7, row);
  const auto x2 = result.number(8, row);
  const auto y2 = result.number(9, row);
  return {
      .region_id = result.text(0, row),
      .label = result.text(1, row),
      .content_markdown = result.text(10, row),
      .content_html = result.text(11, row),
      .page_no = result.int32(2, row),
      .left_percent = x1 / 999.0 * 100.0,
      .top_percent = y1 / 999.0 * 100.0,
      .width_percent = (x2 - x1) / 999.0 * 100.0,
      .height_percent = (y2 - y1) / 999.0 * 100.0,
      .hidden = false,
  };
}

std::vector<std::string> terms_for_query(std::string_view query) {
  std::vector<std::string> terms;
  std::string token;
  for (std::size_t index = 0; index <= query.size(); ++index) {
    const auto ch = index < query.size() ? static_cast<unsigned char>(query[index]) : 0;
    if (index < query.size() && std::isalnum(ch)) {
      token.push_back(static_cast<char>(std::tolower(ch)));
      continue;
    }
    if (!token.empty()) {
      terms.push_back(token);
      token.clear();
    }
  }
  std::sort(terms.begin(), terms.end());
  terms.erase(std::unique(terms.begin(), terms.end()), terms.end());
  return terms;
}

std::string lower_copy(std::string_view text) {
  std::string value(text);
  std::transform(value.begin(), value.end(), value.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return value;
}

std::vector<std::string> sort_scores(const std::map<std::string, int>& scores, std::size_t limit) {
  std::vector<std::pair<std::string, int>> ordered(scores.begin(), scores.end());
  std::sort(ordered.begin(), ordered.end(), [](const auto& left, const auto& right) {
    if (left.second != right.second) {
      return left.second > right.second;
    }
    return left.first < right.first;
  });
  std::vector<std::string> hashes;
  for (const auto& [hash, _] : ordered) {
    if (hashes.size() >= limit) {
      break;
    }
    hashes.push_back(hash);
  }
  return hashes;
}

void load_page_ocr(const WorkbenchRepository::Impl& impl, const std::string& file_hash, StoredPage& page) {
  auto ocr = impl.statement(
      "SELECT raw_text, cleaned_text, status, coalesce(error, '') FROM document_page_ocr "
      "WHERE file_hash = ? AND page_no = ? ORDER BY created_at DESC LIMIT 1");
  ocr.bind_text(1, file_hash);
  ocr.bind_int32(2, page.page_no);
  auto result = ocr.query();
  if (result.rows() == 0) {
    return;
  }
  page.raw_text = result.text(0, 0);
  page.cleaned_text = result.text(1, 0);
  page.status = result.text(2, 0);
  page.error = result.text(3, 0);
}

void load_page_regions(const WorkbenchRepository::Impl& impl, const std::string& file_hash, StoredPage& page) {
  auto regions = impl.statement(
      "SELECT r.region_id, r.label, r.page_no, r.engine_id, r.profile_id, r.bbox_kind, r.x1, r.y1, r.x2, r.y2, "
      "coalesce(a.content_markdown, r.content_markdown, ''), coalesce(a.content_html, r.content_html, '') "
      "FROM document_regions r LEFT JOIN document_region_annotations a ON a.region_id = r.region_id "
      "WHERE r.file_hash = ? AND r.page_no = ? ORDER BY r.source_span_start, r.region_id");
  regions.bind_text(1, file_hash);
  regions.bind_int32(2, page.page_no);
  auto region_rows = regions.query();
  for (idx_t row = 0; row < region_rows.rows(); ++row) {
    page.boxes.push_back(overlay_from_row(region_rows, row));
  }

  auto links = impl.statement(
      "SELECT region_id, text_start, text_end FROM document_text_region_links "
      "WHERE file_hash = ? AND page_no = ? ORDER BY text_start, text_end");
  links.bind_text(1, file_hash);
  links.bind_int32(2, page.page_no);
  auto link_rows = links.query();
  for (idx_t row = 0; row < link_rows.rows(); ++row) {
    page.spans.push_back({link_rows.text(0, row), page.page_no,
                          static_cast<std::size_t>(link_rows.uint64(1, row)),
                          static_cast<std::size_t>(link_rows.uint64(2, row))});
  }
}

std::vector<StoredPage> load_pages(const WorkbenchRepository::Impl& impl, const std::string& file_hash) {
  auto pages = impl.statement(
      "SELECT p.page_no, coalesce(i.path, ''), coalesce(p.width_px, 0), coalesce(p.height_px, 0), "
      "p.render_dpi, p.status, coalesce(p.error, '') "
      "FROM document_pages p LEFT JOIN document_preview_images i "
      "ON i.file_hash = p.file_hash AND i.page_no = p.page_no AND i.variant = 'source' "
      "WHERE p.file_hash = ? ORDER BY p.page_no");
  pages.bind_text(1, file_hash);
  auto page_rows = pages.query();

  std::vector<StoredPage> loaded;
  for (idx_t row = 0; row < page_rows.rows(); ++row) {
    StoredPage page;
    page.page_no = page_rows.int32(0, row);
    page.image_path = page_rows.text(1, row);
    page.width_px = page_rows.int32(2, row);
    page.height_px = page_rows.int32(3, row);
    page.dpi = page_rows.int32(4, row);
    page.status = page_rows.text(5, row);
    page.error = page_rows.text(6, row);
    load_page_ocr(impl, file_hash, page);
    load_page_regions(impl, file_hash, page);
    loaded.push_back(std::move(page));
  }
  return loaded;
}

std::vector<std::string> load_run_hashes(const WorkbenchRepository::Impl& impl, const std::string& run_id) {
  auto query = impl.statement(
      "SELECT DISTINCT file_hash FROM ingest_work_units WHERE run_id = ? ORDER BY file_hash");
  query.bind_text(1, run_id);
  auto result = query.query();
  std::vector<std::string> hashes;
  for (idx_t row = 0; row < result.rows(); ++row) {
    hashes.push_back(result.text(0, row));
  }
  return hashes;
}

}  // namespace

WorkbenchSnapshot WorkbenchRepository::load_snapshot() const {
  std::scoped_lock lock(impl_->mutex);
  WorkbenchSnapshot snapshot;

  auto documents = impl_->query(
      "SELECT f.file_hash, coalesce(l.absolute_path, ''), coalesce(l.relative_path, f.display_name), "
      "f.status, coalesce(f.error, ''), f.size_bytes, f.page_count "
      "FROM files f LEFT JOIN ("
      "  SELECT file_hash, absolute_path, relative_path FROM ("
      "    SELECT file_hash, absolute_path, relative_path, "
      "    row_number() OVER (PARTITION BY file_hash ORDER BY observed_at DESC) AS rn "
      "    FROM file_locations"
      "  ) WHERE rn = 1"
      ") l ON l.file_hash = f.file_hash ORDER BY f.updated_at DESC");
  for (idx_t row = 0; row < documents.rows(); ++row) {
    StoredDocument document;
    document.file_hash = documents.text(0, row);
    document.absolute_path = documents.text(1, row);
    document.relative_path = documents.text(2, row);
    document.status = documents.text(3, row);
    document.error = documents.text(4, row);
    document.size_bytes = documents.uint64(5, row);
    document.page_count = documents.int32(6, row);
    document.pages = load_pages(*impl_, document.file_hash);
    snapshot.documents.push_back(std::move(document));
  }

  auto runs = impl_->query(
      "SELECT run_id, root_path, status, coalesce(error, ''), profile_id, engine_id, model_id, coalesce(runtime_id, ''), "
      "coalesce(queued_files, 0), coalesce(processed_pages, 0), coalesce(total_pages, 0) "
      "FROM ingest_runs ORDER BY started_at DESC LIMIT 50");
  for (idx_t row = 0; row < runs.rows(); ++row) {
    StoredRun run;
    run.run_id = runs.text(0, row);
    run.root_path = runs.text(1, row);
    run.status = runs.text(2, row);
    run.error = runs.text(3, row);
    run.profile_id = runs.text(4, row);
    run.engine_id = runs.text(5, row);
    run.model_id = runs.text(6, row);
    run.runtime_id = runs.text(7, row);
    run.queued_files = runs.int32(8, row);
    run.processed_pages = runs.int32(9, row);
    run.total_pages = runs.int32(10, row);
    run.file_hashes = load_run_hashes(*impl_, run.run_id);
    snapshot.runs.push_back(std::move(run));
  }

  return snapshot;
}

std::vector<std::string> WorkbenchRepository::search_document_hashes(std::string_view query,
                                                                     std::size_t limit) const {
  std::scoped_lock lock(impl_->mutex);
  const auto terms = terms_for_query(query);
  std::map<std::string, int> scores;
  for (const auto& term : terms) {
    auto statement = impl_->statement("SELECT DISTINCT file_hash FROM document_terms WHERE term = ? LIMIT 1000");
    statement.bind_text(1, term);
    auto result = statement.query();
    for (idx_t row = 0; row < result.rows(); ++row) {
      ++scores[result.text(0, row)];
    }
  }
  if (!scores.empty()) {
    return sort_scores(scores, limit);
  }

  const auto needle = "%" + lower_copy(query) + "%";
  auto fallback = impl_->statement(
      "SELECT DISTINCT f.file_hash FROM files f LEFT JOIN document_page_ocr o ON o.file_hash = f.file_hash "
      "WHERE lower(f.display_name) LIKE ? OR lower(o.cleaned_text) LIKE ? ORDER BY f.updated_at DESC LIMIT ?");
  fallback.bind_text(1, needle);
  fallback.bind_text(2, needle);
  fallback.bind_int32(3, static_cast<int>(limit));
  auto result = fallback.query();
  std::vector<std::string> hashes;
  for (idx_t row = 0; row < result.rows(); ++row) {
    hashes.push_back(result.text(0, row));
  }
  return hashes;
}

}  // namespace uocr::storage
