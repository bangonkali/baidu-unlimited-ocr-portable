#include "workbench_state.hpp"

#include <algorithm>
#include <cctype>
#include <sstream>
#include <stdexcept>
#include <string>
#include <thread>
#include <utility>

#include "uocr/app/app_logger.hpp"
#include "uocr/core/ocr_parser.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/ocr/unlimited_ocr_ffi_engine.hpp"
#include "uocr/render/mupdf_page_renderer.hpp"
#include "uocr/render/png_dimensions.hpp"

namespace uocr::server {
namespace {

bool has_extension(const std::filesystem::path& path, std::string_view expected) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == expected;
}

bool is_image_file(const std::filesystem::path& path) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == ".bmp" || ext == ".jpeg" || ext == ".jpg" || ext == ".png" || ext == ".tif" ||
         ext == ".tiff" || ext == ".webp";
}

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

void refresh_document_aggregate(WorkbenchService::Impl::DocumentState& document) {
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

std::string document_status_for(const WorkbenchService::Impl::DocumentState& document) {
  bool completed = false;
  bool failed = false;
  for (const auto& page : document.pages) {
    completed = completed || page.status == "completed";
    failed = failed || page.status == "failed";
  }
  if (completed && failed) {
    return "completed_with_errors";
  }
  return failed ? "failed" : "completed";
}

}  // namespace

std::vector<WorkbenchService::Impl::PageState> WorkbenchService::Impl::prepare_pages(
    const DiscoveredFile& file, const std::string& file_hash) const {
  if (is_image_file(file.absolute_path)) {
    PageState page;
    page.image_path = file.absolute_path;
    try {
      const auto size = read_png_dimensions(file.absolute_path);
      page.width_px = size.width_px;
      page.height_px = size.height_px;
    } catch (const std::exception&) {
      page.width_px = 0;
      page.height_px = 0;
    }
    return {page};
  }

  if (!has_extension(file.absolute_path, ".pdf")) {
    throw std::runtime_error("unsupported input type");
  }

  log_info(logger, "pdf", "rendering " + file.relative_path.generic_string() + " at 200 DPI with MuPDF");
  const auto cache_root = app_root / "cache" / "rendered-pages" / file_hash;
  MupdfPageRenderer renderer;
  const auto rendered_pages = renderer.render_document(file.absolute_path, cache_root, 200);
  std::vector<PageState> pages;
  pages.reserve(rendered_pages.size());
  for (const auto& rendered : rendered_pages) {
    PageState page;
    page.page_no = rendered.page_no;
    page.image_path = rendered.image_path;
    page.width_px = rendered.width_px;
    page.height_px = rendered.height_px;
    page.dpi = rendered.dpi;
    pages.push_back(std::move(page));
  }
  return pages;
}

void WorkbenchService::Impl::start_run(std::string const& run_id,
                                       std::vector<DiscoveredFile> files,
                                       std::string profile_id) {
  std::thread([shared = shared_from_this(), run_id, files = std::move(files), profile_id = std::move(profile_id)]() {
    shared->process_run(run_id, files, profile_id);
  }).detach();
}

void WorkbenchService::Impl::fail_run(const std::string& run_id, const std::string& message) {
  std::scoped_lock lock(mutex);
  auto& run = runs[run_id];
  run.status = "failed";
  run.error = message;
  for (const auto& hash : run.file_hashes) {
    auto& document = documents[hash];
    if (document.status == "queued" || document.status == "running" || document.status == "rendering") {
      document.status = "failed";
      document.error = message;
    }
  }
}

void WorkbenchService::Impl::process_run(const std::string& run_id,
                                         const std::vector<DiscoveredFile>& files,
                                         const std::string& profile_id) {
  const auto* profile = find_ocr_profile(profile_id);
  profile = profile != nullptr ? profile : &default_ocr_profile();
  if (files.empty()) {
    std::scoped_lock lock(mutex);
    runs[run_id].status = "completed";
    log_info(logger, "ingest", "run " + run_id + " finished: no supported files found");
    return;
  }
  if (!model_ready()) {
    fail_run(run_id, "model assets are missing; open Models and download Unlimited-OCR Q4_K_M");
    log_error(logger, "ingest", "run " + run_id + " failed because model assets are missing");
    return;
  }
  if (!std::filesystem::exists(ffi_path())) {
    fail_run(run_id, "uocr-ffi runtime is missing: " + ffi_path().string());
    log_error(logger, "ingest", "run " + run_id + " failed because uocr-ffi is missing");
    return;
  }

  log_info(logger, "models", "loading CUDA Unlimited-OCR runtime from " + ffi_path().string());
  UnlimitedOcrFfiEngine engine({ffi_path(), model_path(), mmproj_path()}, *profile);
  bool any_failed = false;
  {
    std::scoped_lock lock(mutex);
    runs[run_id].status = "running";
  }
  log_info(logger, "ingest", "run " + run_id + " started with " + std::to_string(files.size()) + " files");

  for (const auto& file : files) {
    const auto hash = stable_hash(file);
    {
      std::scoped_lock lock(mutex);
      if (runs[run_id].cancel_requested) {
        runs[run_id].status = "cancelled";
        return;
      }
      documents[hash].status = has_extension(file.absolute_path, ".pdf") ? "rendering" : "running";
    }

    std::vector<PageState> pages;
    try {
      pages = prepare_pages(file, hash);
    } catch (const std::exception& error) {
      std::scoped_lock lock(mutex);
      auto& document = documents[hash];
      document.status = "failed";
      document.error = error.what();
      runs[run_id].processed_pages += 1;
      any_failed = true;
      log_error(logger, "pdf", file.relative_path.generic_string() + " failed: " + error.what());
      continue;
    }

    {
      std::scoped_lock lock(mutex);
      auto& run = runs[run_id];
      auto& document = documents[hash];
      document.pages = pages;
      document.status = "running";
      run.total_pages += std::max(0, static_cast<int>(pages.size()) - 1);
    }

    for (std::size_t index = 0; index < pages.size(); ++index) {
      const auto& page = pages[index];
      {
        std::scoped_lock lock(mutex);
        if (runs[run_id].cancel_requested) {
          runs[run_id].status = "cancelled";
          return;
        }
        documents[hash].pages[index].status = "running";
      }
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

      std::scoped_lock lock(mutex);
      auto& document = documents[hash];
      auto& page_state = document.pages[index];
      if (!error.empty()) {
        any_failed = true;
        page_state.status = "failed";
        page_state.error = error;
        log_error(logger, "ocr", page_label(file, page.page_no, static_cast<int>(pages.size())) + " failed: " + error);
      } else {
        const auto parsed = parse_ocr_markers(result.text, {.file_hash = hash, .page_no = page.page_no});
        page_state.raw_text = result.text;
        page_state.cleaned_text = parsed.cleaned_text.empty() ? result.text : parsed.cleaned_text;
        page_state.boxes = to_overlay_boxes(parsed, page.page_no);
        page_state.spans = parsed.text_region_spans;
        page_state.status = "completed";
        log_info(logger, "ocr", page_label(file, page.page_no, static_cast<int>(pages.size())) + " completed");
      }
      refresh_document_aggregate(document);
      runs[run_id].processed_pages += 1;
    }

    std::scoped_lock lock(mutex);
    auto& document = documents[hash];
    document.status = document_status_for(document);
  }

  std::scoped_lock lock(mutex);
  auto& run = runs[run_id];
  run.status = any_failed ? "completed_with_errors" : "completed";
  log_info(logger, "ingest", "run " + run_id + " finished with status " + run.status);
}

}  // namespace uocr::server
