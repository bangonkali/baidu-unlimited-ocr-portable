#include "workbench_repository_impl.hpp"

#include <algorithm>
#include <cctype>

namespace uocr::storage {
namespace {

constexpr std::string_view kEngine = "unlimited-ocr";
constexpr std::string_view kProfile = "experimental-exact-prefill-q4";
constexpr std::string_view kModel = "unlimited-ocr-q4-k-m";

std::string extension_for(const StoredDocument& document) {
  auto ext = document.absolute_path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext;
}

void bind_nullable_text(Statement& statement, idx_t index, std::string_view value) {
  if (value.empty()) {
    statement.bind_null(index);
  } else {
    statement.bind_text(index, value);
  }
}

}  // namespace

void WorkbenchRepository::upsert_run(const StoredRun& run) {
  std::scoped_lock lock(impl_->mutex);
  auto statement = impl_->statement(
      "INSERT INTO ingest_runs(run_id, root_path, status, profile_id, engine_id, model_id, reprocess, error, "
      "queued_files, processed_pages, total_pages, finished_at) "
      "VALUES (?, ?, ?, ?, ?, ?, false, ?, ?, ?, ?, CASE WHEN ? IN "
      "('completed','completed_with_errors','failed','cancelled') THEN current_timestamp ELSE NULL END) "
      "ON CONFLICT(run_id) DO UPDATE SET status=excluded.status, error=excluded.error, "
      "queued_files=excluded.queued_files, processed_pages=excluded.processed_pages, "
      "total_pages=excluded.total_pages, model_id=excluded.model_id, finished_at=excluded.finished_at");
  statement.bind_text(1, run.run_id);
  statement.bind_text(2, run.root_path);
  statement.bind_text(3, run.status);
  statement.bind_text(4, run.profile_id.empty() ? kProfile : std::string_view(run.profile_id));
  statement.bind_text(5, run.engine_id.empty() ? kEngine : std::string_view(run.engine_id));
  statement.bind_text(6, run.model_id.empty() ? kModel : std::string_view(run.model_id));
  bind_nullable_text(statement, 7, run.error);
  statement.bind_int32(8, run.queued_files);
  statement.bind_int32(9, run.processed_pages);
  statement.bind_int32(10, run.total_pages);
  statement.bind_text(11, run.status);
  statement.execute();
}

void WorkbenchRepository::put_setting_string(std::string_view key, std::string_view value) {
  std::scoped_lock lock(impl_->mutex);
  const auto quoted_key = quote_sql_string(key);
  impl_->execute("BEGIN TRANSACTION");
  try {
    impl_->execute("DELETE FROM settings WHERE key = " + quoted_key);
    impl_->execute("INSERT INTO settings(key, value, updated_at) VALUES (" + quoted_key + ", " +
                   quote_sql_string(escape_json_string(value)) + "::JSON, current_timestamp)");
    impl_->execute("COMMIT");
  } catch (...) {
    impl_->execute("ROLLBACK");
    throw;
  }
}

void WorkbenchRepository::upsert_document(const StoredDocument& document, std::string_view root_path) {
  std::scoped_lock lock(impl_->mutex);
  auto delete_file = impl_->statement("DELETE FROM files WHERE file_hash = ?");
  delete_file.bind_text(1, document.file_hash);
  delete_file.execute();

  auto file = impl_->statement(
      "INSERT INTO files(file_hash, display_name, extension, size_bytes, page_count, status, error, updated_at) "
      "VALUES (?, ?, ?, ?, ?, ?, ?, current_timestamp)");
  file.bind_text(1, document.file_hash);
  file.bind_text(2, document.relative_path.filename().string());
  file.bind_text(3, extension_for(document));
  file.bind_uint64(4, document.size_bytes);
  file.bind_int32(5, document.page_count);
  file.bind_text(6, document.status);
  bind_nullable_text(file, 7, document.error);
  file.execute();

  auto delete_location = impl_->statement("DELETE FROM file_locations WHERE file_hash = ? AND absolute_path = ?");
  delete_location.bind_text(1, document.file_hash);
  delete_location.bind_text(2, document.absolute_path.string());
  delete_location.execute();

  auto location = impl_->statement(
      "INSERT INTO file_locations(file_hash, root_path, absolute_path, relative_path, observed_at) "
      "VALUES (?, ?, ?, ?, current_timestamp)");
  location.bind_text(1, document.file_hash);
  location.bind_text(2, root_path);
  location.bind_text(3, document.absolute_path.string());
  location.bind_text(4, document.relative_path.generic_string());
  location.execute();
}

void WorkbenchRepository::upsert_page(const std::string& file_hash, const StoredPage& page) {
  std::scoped_lock lock(impl_->mutex);
  auto delete_page = impl_->statement("DELETE FROM document_pages WHERE file_hash = ? AND page_no = ?");
  delete_page.bind_text(1, file_hash);
  delete_page.bind_int32(2, page.page_no);
  delete_page.execute();

  auto statement = impl_->statement(
      "INSERT INTO document_pages(file_hash, page_no, width_px, height_px, render_dpi, status, error) "
      "VALUES (?, ?, ?, ?, ?, ?, ?)");
  statement.bind_text(1, file_hash);
  statement.bind_int32(2, page.page_no);
  statement.bind_int32(3, page.width_px);
  statement.bind_int32(4, page.height_px);
  statement.bind_int32(5, page.dpi);
  statement.bind_text(6, page.status);
  bind_nullable_text(statement, 7, page.error);
  statement.execute();

  if (!page.image_path.empty() && page.width_px > 0 && page.height_px > 0) {
    auto delete_preview = impl_->statement(
        "DELETE FROM document_preview_images WHERE file_hash = ? AND page_no = ? AND variant = 'source'");
    delete_preview.bind_text(1, file_hash);
    delete_preview.bind_int32(2, page.page_no);
    delete_preview.execute();

    auto preview = impl_->statement(
        "INSERT INTO document_preview_images(file_hash, page_no, variant, path, width_px, height_px) "
        "VALUES (?, ?, 'source', ?, ?, ?)");
    preview.bind_text(1, file_hash);
    preview.bind_int32(2, page.page_no);
    preview.bind_text(3, page.image_path.string());
    preview.bind_int32(4, page.width_px);
    preview.bind_int32(5, page.height_px);
    preview.execute();
  }
}

}  // namespace uocr::storage
