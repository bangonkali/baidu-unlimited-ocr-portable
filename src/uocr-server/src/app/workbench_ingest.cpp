#include "workbench_state.hpp"

#include <algorithm>
#include <sstream>
#include <string>

#include "uocr/app/app_logger.hpp"
#include "uocr/core/ocr_parser.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/ocr/unlimited_ocr_ffi_engine.hpp"

namespace uocr::server {
namespace {

void log_info(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->info(component, message);
  }
}

void log_error(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->error(component, message);
  }
}

std::string page_label(const DiscoveredFile& file, int page_no, int page_count) {
  std::ostringstream out;
  out << file.relative_path.generic_string() << " page " << page_no << "/" << page_count;
  return out.str();
}

}  // namespace

void WorkbenchService::Impl::process_run(const std::string& run_id,
                                         const std::vector<DiscoveredFile>& files,
                                         const std::string& profile_id,
                                         const std::string& model_id) {
  const auto* profile = find_ocr_profile(profile_id);
  profile = profile != nullptr ? profile : &default_ocr_profile();
  const auto* model_entry = find_model_catalog_entry(model_id);
  if (model_entry == nullptr) {
    fail_run(run_id, "unknown model id: " + model_id);
    log_error(logger, "ingest", "run " + run_id + " failed because model id is unknown: " + model_id);
    return;
  }
  if (files.empty()) {
    Json::Value run_event;
    {
      std::scoped_lock lock(mutex);
      runs[run_id].status = "completed";
      persist_run(runs[run_id]);
      persist_diagnostic(run_id, "info", "run finished with no supported files");
      run_event = run_record(runs[run_id]);
    }
    log_info(logger, "ingest", "run " + run_id + " finished: no supported files found");
    publish_event("run.changed", run_event);
    publish_status_changed();
    return;
  }
  if (!model_ready(model_id)) {
    fail_run(run_id, "model assets are missing; open Models and download " + std::string(model_entry->display_name));
    log_error(logger, "ingest", "run " + run_id + " failed because model assets are missing");
    return;
  }
  if (!std::filesystem::exists(ffi_path())) {
    fail_run(run_id, "uocr-ffi runtime is missing: " + ffi_path().string());
    log_error(logger, "ingest", "run " + run_id + " failed because uocr-ffi is missing");
    return;
  }

  log_info(logger, "models", "loading CUDA Unlimited-OCR runtime model=" + model_id +
                                " file=" + std::string(model_entry->model_file));
  UnlimitedOcrFfiEngine engine({ffi_path(), model_path(model_id), mmproj_path()}, *profile);
  bool any_failed = false;
  Json::Value run_event;
  {
    std::scoped_lock lock(mutex);
    runs[run_id].status = "running";
    persist_run(runs[run_id]);
    persist_diagnostic(run_id, "info", "run started with " + std::to_string(files.size()) +
                                             " files using " + model_id);
    run_event = run_record(runs[run_id]);
  }
  publish_event("run.changed", run_event);
  publish_status_changed();
  log_info(logger, "ingest", "run " + run_id + " started with " + std::to_string(files.size()) + " files");

  for (const auto& file : files) {
    const auto hash = stable_hash(file);
    Json::Value document_event;
    run_event = Json::Value();
    {
      std::scoped_lock lock(mutex);
      if (runs[run_id].cancel_requested) {
        runs[run_id].status = "cancelled";
        persist_run(runs[run_id]);
        run_event = run_record(runs[run_id]);
      } else {
        documents[hash].status = lower(file.absolute_path.extension().string()) == ".pdf" ? "rendering" : "running";
        persist_document(documents[hash], runs[run_id].root_path);
        document_event = document_summary(documents[hash]);
      }
    }
    if (!run_event.isNull() && document_event.isNull()) {
      publish_event("run.changed", run_event);
      publish_status_changed();
      return;
    }
    publish_event("document.changed", document_event);

    std::vector<PageState> pages;
    try {
      pages = prepare_pages(file, hash);
    } catch (const std::exception& error) {
      {
        std::scoped_lock lock(mutex);
        auto& document = documents[hash];
        document.status = "failed";
        document.error = error.what();
        runs[run_id].processed_pages += 1;
        persist_document(document, runs[run_id].root_path);
        persist_run(runs[run_id]);
        persist_work_unit(run_id, hash, 1, "failed", 1, error.what());
        persist_diagnostic(run_id, "error", file.relative_path.generic_string() + " failed: " + error.what());
        document_event = document_summary(document);
        run_event = run_record(runs[run_id]);
      }
      any_failed = true;
      log_error(logger, "pdf", file.relative_path.generic_string() + " failed: " + error.what());
      publish_event("document.changed", document_event);
      publish_event("run.changed", run_event);
      publish_status_changed();
      continue;
    }

    std::vector<Json::Value> page_events;
    {
      std::scoped_lock lock(mutex);
      auto& run = runs[run_id];
      auto& document = documents[hash];
      document.pages = pages;
      document.status = "running";
      run.total_pages += std::max(0, static_cast<int>(pages.size()) - 1);
      persist_document(document, run.root_path);
      persist_run(run);
      document_event = document_summary(document);
      run_event = run_record(run);
      for (const auto& page : document.pages) {
        persist_page(hash, page);
        persist_work_unit(run_id, hash, page.page_no, page.status, 0, page.error);
        page_events.push_back(document_page_record(document, page));
      }
    }
    publish_event("document.changed", document_event);
    publish_event("run.changed", run_event);
    for (const auto& page_event : page_events) {
      publish_event("document.page.changed", page_event);
    }
    publish_status_changed();

    for (std::size_t index = 0; index < pages.size(); ++index) {
      const auto& page = pages[index];
      Json::Value page_event;
      std::vector<Json::Value> cancel_page_events;
      bool cancelled = false;
      {
        std::scoped_lock lock(mutex);
        if (runs[run_id].cancel_requested) {
          auto& run = runs[run_id];
          auto& document = documents[hash];
          run.status = "cancelled";
          document.status = "cancelled";
          for (auto& page_state : document.pages) {
            if (page_state.status == "queued" || page_state.status == "running") {
              page_state.status = "cancelled";
            }
            persist_page(hash, page_state);
            persist_work_unit(run_id, hash, page_state.page_no, page_state.status, 0, page_state.error);
            cancel_page_events.push_back(document_page_record(document, page_state));
          }
          persist_document(document, run.root_path);
          persist_run(run);
          persist_diagnostic(run_id, "warn", "run cancelled");
          run_event = run_record(run);
          document_event = document_summary(document);
          cancelled = true;
        } else {
          documents[hash].pages[index].status = "running";
          persist_page(hash, documents[hash].pages[index]);
          persist_work_unit(run_id, hash, documents[hash].pages[index].page_no, "running", 1, "");
          page_event = document_page_record(documents[hash], documents[hash].pages[index]);
        }
      }
      if (cancelled) {
        publish_event("run.changed", run_event);
        publish_event("document.changed", document_event);
        for (const auto& item : cancel_page_events) {
          publish_event("document.page.changed", item);
        }
        publish_status_changed();
        return;
      }
      publish_event("document.page.changed", page_event);
      log_info(logger, "ocr", "processing " + page_label(file, page.page_no, static_cast<int>(pages.size())));

      OcrResult result;
      std::string error;
      try {
        result = engine.recognize_image({page.image_path, "document parsing.", profile->default_max_tokens},
                                        [](const OcrEvent&) {});
        if (!result.ok) {
          error = result.error.empty() ? "OCR failed" : result.error;
        }
      } catch (const std::exception& exception) {
        error = exception.what();
      }

      Json::Value regions_event;
      Json::Value text_event;
      {
        std::scoped_lock lock(mutex);
        auto& document = documents[hash];
        auto& page_state = document.pages[index];
        if (!error.empty()) {
          any_failed = true;
          page_state.status = "failed";
          page_state.error = error;
        } else {
          const auto parsed = parse_ocr_markers(result.text,
                                                {.file_hash = hash,
                                                 .page_no = page.page_no,
                                                 .engine_id = "unlimited-ocr",
                                                 .profile_id = profile->key});
          page_state.raw_text = result.text;
          page_state.cleaned_text = parsed.cleaned_text.empty() ? result.text : parsed.cleaned_text;
          page_state.boxes = to_overlay_boxes(parsed, page.page_no);
          page_state.spans = parsed.text_region_spans;
          apply_region_content(page_state);
          page_state.status = "completed";
        }
        refresh_document_aggregate(document);
        runs[run_id].processed_pages += 1;
        persist_page_ocr(hash, page_state, profile->key);
        persist_work_unit(run_id, hash, page_state.page_no, page_state.status, 1, page_state.error);
        persist_document(document, runs[run_id].root_path);
        persist_run(runs[run_id]);
        if (!error.empty()) {
          persist_diagnostic(run_id, "error",
                             page_label(file, page.page_no, static_cast<int>(pages.size())) + " failed: " + error);
        }
        page_event = document_page_record(document, page_state);
        document_event = document_summary(document);
        regions_event = document_regions_record(document);
        text_event = document_text_record(document);
        run_event = run_record(runs[run_id]);
      }
      if (!error.empty()) {
        log_error(logger, "ocr", page_label(file, page.page_no, static_cast<int>(pages.size())) + " failed: " + error);
      } else {
        log_info(logger, "ocr", page_label(file, page.page_no, static_cast<int>(pages.size())) + " completed");
      }
      publish_event("document.page.changed", page_event);
      publish_event("document.changed", document_event);
      publish_event("document.regions.changed", regions_event);
      publish_event("document.text.changed", text_event);
      publish_event("run.changed", run_event);
      publish_status_changed();
    }

    {
      std::scoped_lock lock(mutex);
      auto& document = documents[hash];
      document.status = document_status_for(document);
      persist_document(document, runs[run_id].root_path);
      document_event = document_summary(document);
    }
    publish_event("document.changed", document_event);
  }

  std::string final_status;
  {
    std::scoped_lock lock(mutex);
    auto& run = runs[run_id];
    run.status = any_failed ? "completed_with_errors" : "completed";
    final_status = run.status;
    persist_run(run);
    persist_diagnostic(run_id, any_failed ? "warn" : "info", "run finished with status " + final_status);
    run_event = run_record(run);
  }
  log_info(logger, "ingest", "run " + run_id + " finished with status " + final_status);
  publish_event("run.changed", run_event);
  publish_status_changed();
}

}  // namespace uocr::server
