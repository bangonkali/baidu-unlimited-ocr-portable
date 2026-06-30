#include "workbench_state.hpp"

#include <filesystem>
#include <string_view>

#include "uocr/app/app_logger.hpp"
#include "uocr/core/profiles.hpp"

namespace uocr::server {
namespace {

storage::StoredPage stored_page(const WorkbenchService::Impl::PageState& page) {
  storage::StoredPage stored;
  stored.page_no = page.page_no;
  stored.image_path = page.image_path;
  stored.width_px = page.width_px;
  stored.height_px = page.height_px;
  stored.dpi = page.dpi;
  stored.status = page.status;
  stored.error = page.error;
  stored.raw_text = page.raw_text;
  stored.cleaned_text = page.cleaned_text;
  stored.boxes = page.boxes;
  stored.spans = page.spans;
  return stored;
}

storage::StoredDocument stored_document(const WorkbenchService::Impl::DocumentState& document) {
  storage::StoredDocument stored;
  stored.file_hash = document.file_hash;
  stored.absolute_path = document.absolute_path;
  stored.relative_path = document.relative_path;
  stored.status = document.status;
  stored.error = document.error;
  stored.page_count = document.pages.empty() ? 1 : static_cast<int>(document.pages.size());
  std::error_code error;
  if (std::filesystem::exists(document.absolute_path, error)) {
    stored.size_bytes = static_cast<std::uint64_t>(std::filesystem::file_size(document.absolute_path, error));
  }
  for (const auto& page : document.pages) {
    stored.pages.push_back(stored_page(page));
  }
  return stored;
}

storage::StoredRun stored_run(const WorkbenchService::Impl::RunState& run) {
  storage::StoredRun stored;
  stored.run_id = run.run_id;
  stored.root_path = run.root_path;
  stored.status = run.status;
  stored.error = run.error;
  stored.profile_id = run.profile_id;
  stored.engine_id = run.engine_id;
  stored.queued_files = run.queued_files;
  stored.processed_pages = run.processed_pages;
  stored.total_pages = run.total_pages;
  stored.model_id = run.model_id;
  stored.runtime_id = run.runtime_id;
  stored.file_hashes = run.file_hashes;
  return stored;
}

void refresh_document(WorkbenchService::Impl::DocumentState& document) {
  document.raw_text.clear();
  document.cleaned_text.clear();
  document.boxes.clear();
  document.spans.clear();
  for (const auto& page : document.pages) {
    if (!document.raw_text.empty()) {
      document.raw_text += "\n\n";
      document.cleaned_text += "\n\n";
    }
    document.raw_text += page.raw_text;
    document.cleaned_text += page.cleaned_text;
    document.boxes.insert(document.boxes.end(), page.boxes.begin(), page.boxes.end());
    document.spans.insert(document.spans.end(), page.spans.begin(), page.spans.end());
  }
}

WorkbenchService::Impl::PageState page_state(const storage::StoredPage& stored) {
  WorkbenchService::Impl::PageState page;
  page.page_no = stored.page_no;
  page.image_path = stored.image_path;
  page.width_px = stored.width_px;
  page.height_px = stored.height_px;
  page.dpi = stored.dpi;
  page.status = stored.status;
  page.error = stored.error;
  page.raw_text = stored.raw_text;
  page.cleaned_text = stored.cleaned_text;
  page.boxes = stored.boxes;
  page.spans = stored.spans;
  return page;
}

}  // namespace

void WorkbenchService::Impl::load_persisted_snapshot() {
  try {
    repository = std::make_shared<storage::WorkbenchRepository>(app_root / "data" / "uocr.duckdb");
    const auto selected = repository->setting_string("selected_model_id", default_model_id());
    selected_model_id = find_model_catalog_entry(selected) != nullptr ? selected : std::string(default_model_id());
    const auto selected_profile = repository->setting_string("selected_profile_id", selected_profile_id);
    selected_profile_id = find_ocr_profile(selected_profile) != nullptr ? selected_profile : default_ocr_profile().key;
    selected_runtime_id = repository->setting_string("selected_runtime_id", "");
    const auto snapshot = repository->load_snapshot();
    for (const auto& stored : snapshot.documents) {
      DocumentState document;
      document.file_hash = stored.file_hash;
      document.absolute_path = stored.absolute_path;
      document.relative_path = stored.relative_path;
      document.status = stored.status;
      document.error = stored.error;
      for (const auto& stored_page : stored.pages) {
        document.pages.push_back(page_state(stored_page));
      }
      refresh_document(document);
      documents[document.file_hash] = std::move(document);
    }
    for (const auto& stored : snapshot.runs) {
      RunState run;
      run.run_id = stored.run_id;
      run.root_path = stored.root_path;
      run.status = stored.status;
      run.error = stored.error;
      run.profile_id = stored.profile_id;
      run.engine_id = stored.engine_id;
      run.queued_files = stored.queued_files;
      run.processed_pages = stored.processed_pages;
      run.total_pages = stored.total_pages;
      run.model_id = stored.model_id.empty() ? std::string(default_model_id()) : stored.model_id;
      run.runtime_id = stored.runtime_id;
      run.file_hashes = stored.file_hashes;
      runs[run.run_id] = std::move(run);
    }
    if (logger) {
      logger->info("database", "opened DuckDB at " + repository->database_path().string() + " with " +
                                   std::to_string(documents.size()) + " persisted documents");
    }
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("DuckDB persistence disabled: ") + error.what());
    }
    repository.reset();
  }
}

void WorkbenchService::Impl::persist_selected_model() const {
  if (!repository) {
    return;
  }
  try {
    repository->put_setting_string("selected_model_id", selected_model_id);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist selected model: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_selected_runtime() const {
  if (!repository) {
    return;
  }
  try {
    repository->put_setting_string("selected_runtime_id", selected_runtime_id);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist selected runtime: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_selected_profile() const {
  if (!repository) {
    return;
  }
  try {
    repository->put_setting_string("selected_profile_id", selected_profile_id);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist selected profile: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_run(const RunState& run) const {
  if (!repository) {
    return;
  }
  try {
    repository->upsert_run(stored_run(run));
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist run: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_document(const DocumentState& document, std::string_view root_path) const {
  if (!repository) {
    return;
  }
  try {
    repository->upsert_document(stored_document(document), root_path);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist document: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_page(const std::string& file_hash, const PageState& page) const {
  if (!repository) {
    return;
  }
  try {
    repository->upsert_page(file_hash, stored_page(page));
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist page: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_page_ocr(const std::string& file_hash,
                                              const PageState& page,
                                              std::string_view profile_id) const {
  if (!repository) {
    return;
  }
  try {
    const auto stored = stored_page(page);
    repository->upsert_page(file_hash, stored);
    repository->replace_page_ocr(file_hash, stored, "unlimited-ocr", profile_id);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist OCR page: ") + error.what());
    }
  }
}

void WorkbenchService::Impl::persist_work_unit(const std::string& run_id,
                                               const std::string& file_hash,
                                               int page_no,
                                               std::string_view status,
                                               int attempts,
                                               std::string_view error) const {
  if (!repository) {
    return;
  }
  try {
    repository->upsert_work_unit(run_id, file_hash, page_no, status, attempts, error);
  } catch (const std::exception& exception) {
    if (logger) {
      logger->error("database", std::string("failed to persist work unit: ") + exception.what());
    }
  }
}

void WorkbenchService::Impl::persist_diagnostic(const std::string& run_id,
                                                std::string_view level,
                                                std::string_view message) const {
  if (!repository) {
    return;
  }
  try {
    repository->append_diagnostic_event(run_id, level, message);
  } catch (const std::exception& error) {
    if (logger) {
      logger->error("database", std::string("failed to persist diagnostic event: ") + error.what());
    }
  }
}

}  // namespace uocr::server
