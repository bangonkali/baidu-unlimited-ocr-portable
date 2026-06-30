#include <cassert>
#include <filesystem>

#include "uocr/storage/workbench_repository.hpp"

namespace {

uocr::storage::StoredDocument sample_document() {
  uocr::storage::StoredPage page;
  page.page_no = 1;
  page.image_path = "cache/page-1.png";
  page.width_px = 1000;
  page.height_px = 1400;
  page.status = "completed";
  page.raw_text = "<|ref|>Invoice total<|/ref|><|det|>[[10,20,100,120]]<|/det|>";
  page.cleaned_text = "Invoice total";
  page.boxes.push_back({
      .region_id = "reg_total",
      .label = "Invoice total",
      .content_markdown = "Invoice total",
      .content_html = "",
      .page_no = 1,
      .left_percent = 1.0,
      .top_percent = 2.0,
      .width_percent = 9.0,
      .height_percent = 10.0,
      .hidden = false,
  });
  page.spans.push_back({"reg_total", 1, 0, 13});

  uocr::storage::StoredDocument document;
  document.file_hash = "file_abc";
  document.absolute_path = "C:/samples/invoice.pdf";
  document.relative_path = "invoice.pdf";
  document.status = "completed";
  document.size_bytes = 1024;
  document.page_count = 1;
  document.pages.push_back(std::move(page));
  return document;
}

void persist_document(uocr::storage::WorkbenchRepository& repository) {
  uocr::storage::StoredRun run;
  run.run_id = "run_storage_test";
  run.root_path = "C:/samples";
  run.status = "completed";
  run.queued_files = 1;
  run.processed_pages = 1;
  run.total_pages = 1;
  run.file_hashes.push_back("file_abc");
  repository.upsert_run(run);

  auto document = sample_document();
  repository.upsert_document(document, run.root_path);
  repository.upsert_page(document.file_hash, document.pages.front());
  repository.replace_page_ocr(document.file_hash, document.pages.front(), "unlimited-ocr", "best-zero-empty-q4");
  repository.upsert_work_unit(run.run_id, document.file_hash, 1, "completed", 1, "");
}

}  // namespace

int main() {
  const auto root = std::filesystem::temp_directory_path() / "uocr_storage_test";
  std::filesystem::remove_all(root);
  std::filesystem::create_directories(root);
  const auto db_path = root / "uocr.duckdb";

  {
    uocr::storage::WorkbenchRepository repository(db_path);
    persist_document(repository);
  }

  uocr::storage::WorkbenchRepository reopened(db_path);
  const auto snapshot = reopened.load_snapshot();
  assert(snapshot.documents.size() == 1);
  assert(snapshot.runs.size() == 1);
  const auto& document = snapshot.documents.front();
  assert(document.file_hash == "file_abc");
  assert(document.pages.size() == 1);
  assert(document.pages.front().cleaned_text == "Invoice total");
  assert(document.pages.front().boxes.size() == 1);
  assert(document.pages.front().boxes.front().content_markdown == "Invoice total");
  assert(document.pages.front().spans.front().region_id == "reg_total");

  const auto results = reopened.search_document_hashes("invoice", 10);
  assert(results.size() == 1);
  assert(results.front() == "file_abc");
  std::filesystem::remove_all(root);
  return 0;
}
