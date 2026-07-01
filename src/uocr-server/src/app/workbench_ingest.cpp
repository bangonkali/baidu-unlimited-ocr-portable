#include "workbench_state.hpp"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <ctime>
#include <iomanip>
#include <map>
#include <optional>
#include <set>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

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

std::string utc_timestamp() {
  const auto now = std::chrono::system_clock::now();
  const auto time = std::chrono::system_clock::to_time_t(now);
  std::tm utc{};
#ifdef _WIN32
  gmtime_s(&utc, &time);
#else
  gmtime_r(&time, &utc);
#endif
  std::ostringstream stream;
  stream << std::put_time(&utc, "%Y-%m-%dT%H:%M:%SZ");
  return stream.str();
}

std::uint64_t elapsed_ms_since(std::chrono::steady_clock::time_point start,
                               std::chrono::steady_clock::time_point stop) {
  return static_cast<std::uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(stop - start).count());
}

Json::Value box_json(const OverlayBox& box) {
  Json::Value value;
  value["region_id"] = box.region_id;
  value["label"] = box.label;
  value["content_markdown"] = box.content_markdown;
  value["content_html"] = box.content_html;
  value["page_no"] = box.page_no;
  value["left_percent"] = box.left_percent;
  value["top_percent"] = box.top_percent;
  value["width_percent"] = box.width_percent;
  value["height_percent"] = box.height_percent;
  value["hidden"] = box.hidden;
  return value;
}

Json::Value span_json(const TextRegionSpan& span) {
  Json::Value value;
  value["region_id"] = span.region_id;
  value["page_no"] = span.page_no;
  value["start"] = static_cast<Json::UInt64>(span.start);
  value["end"] = static_cast<Json::UInt64>(span.end);
  return value;
}

bool same_box(const OverlayBox& left, const OverlayBox& right) {
  return left.region_id == right.region_id && left.label == right.label &&
         left.content_markdown == right.content_markdown && left.content_html == right.content_html &&
         left.page_no == right.page_no && std::abs(left.left_percent - right.left_percent) < 0.0001 &&
         std::abs(left.top_percent - right.top_percent) < 0.0001 &&
         std::abs(left.width_percent - right.width_percent) < 0.0001 &&
         std::abs(left.height_percent - right.height_percent) < 0.0001 && left.hidden == right.hidden;
}

bool same_span(const TextRegionSpan& left, const TextRegionSpan& right) {
  return left.region_id == right.region_id && left.page_no == right.page_no && left.start == right.start &&
         left.end == right.end;
}

class PageStreamPublisher {
 public:
  PageStreamPublisher(WorkbenchService::Impl& service,
                      std::string run_id,
                      std::string file_hash,
                      std::size_t page_index,
                      int page_no,
                      std::string profile_id,
                      std::string model_id,
                      RuntimeVariant runtime)
      : service_(service),
        run_id_(std::move(run_id)),
        file_hash_(std::move(file_hash)),
        page_index_(page_index),
        page_no_(page_no),
        profile_id_(std::move(profile_id)),
        model_id_(std::move(model_id)),
        runtime_(std::move(runtime)),
        started_(std::chrono::steady_clock::now()),
        started_at_(utc_timestamp()) {}

  void start() const {
    Json::Value payload = context_payload();
    payload["started_at"] = started_at_;
    service_.publish_event("ocr.page.stream.started", payload);
  }

  void on_event(const OcrEvent& event) {
    if (event.kind != OcrEvent::Kind::Token || event.text.empty()) {
      return;
    }
    const auto now = std::chrono::steady_clock::now();
    if (!first_token_.has_value()) {
      first_token_ = now;
      first_token_at_ = utc_timestamp();
      first_token_latency_ms_ = elapsed_ms_since(started_, now);
    }

    const auto raw_start = raw_text_.size();
    raw_text_ += event.text;
    const auto raw_end = raw_text_.size();
    chunk_count_ += 1;
    token_count_ = std::max<std::uint64_t>(token_count_, event.index + 1);
    update_rates(now);

    service_.publish_event("ocr.page.raw.delta", raw_delta_payload(event, raw_start, raw_end));
    parse_and_publish();
    service_.publish_event("ocr.page.metrics.changed", metrics_payload("running", ""));
  }

  storage::OcrPageMetrics finish_metrics(std::string_view status, std::string_view error) {
    completed_at_ = utc_timestamp();
    update_rates(std::chrono::steady_clock::now());
    storage::OcrPageMetrics metrics;
    metrics.run_id = run_id_;
    metrics.file_hash = file_hash_;
    metrics.page_no = page_no_;
    metrics.engine_id = "unlimited-ocr";
    metrics.profile_id = profile_id_;
    metrics.model_id = model_id_;
    metrics.runtime_id = runtime_.runtime_id;
    metrics.runtime_platform = runtime_.platform;
    metrics.accelerator = runtime_.accelerator;
    metrics.status = std::string(status);
    metrics.error = std::string(error);
    metrics.token_count = token_count_;
    metrics.chunk_count = chunk_count_;
    metrics.first_token_latency_ms = first_token_latency_ms_;
    metrics.generation_duration_ms = generation_duration_ms_;
    metrics.elapsed_ms = elapsed_ms_;
    metrics.min_tps = min_tps_.has_value() ? *min_tps_ : 0.0;
    metrics.max_tps = max_tps_;
    metrics.avg_tps = avg_tps_;
    metrics.started_at = started_at_;
    metrics.first_token_at = first_token_at_;
    metrics.completed_at = completed_at_;
    return metrics;
  }

  void publish_terminal(std::string_view status, std::string_view error) const {
    service_.publish_event(error.empty() ? "ocr.page.stream.completed" : "ocr.page.stream.failed",
                           metrics_payload(status, error));
  }

 private:
  Json::Value context_payload() const {
    Json::Value payload;
    payload["run_id"] = run_id_;
    payload["file_hash"] = file_hash_;
    payload["page_no"] = page_no_;
    payload["engine_id"] = "unlimited-ocr";
    payload["profile_id"] = profile_id_;
    payload["model_id"] = model_id_;
    payload["runtime_id"] = runtime_.runtime_id;
    payload["runtime_platform"] = runtime_.platform;
    payload["accelerator"] = runtime_.accelerator;
    return payload;
  }

  Json::Value raw_delta_payload(const OcrEvent& event, std::size_t raw_start, std::size_t raw_end) const {
    Json::Value payload = context_payload();
    payload["token_index"] = static_cast<Json::UInt64>(event.index);
    payload["delta"] = event.text;
    payload["raw_start"] = static_cast<Json::UInt64>(raw_start);
    payload["raw_end"] = static_cast<Json::UInt64>(raw_end);
    payload["elapsed_ms"] = static_cast<Json::UInt64>(elapsed_ms_);
    payload["avg_tps"] = avg_tps_;
    return payload;
  }

  Json::Value text_patch_payload(std::string_view op, std::size_t start, std::size_t end, std::string text) const {
    Json::Value payload = context_payload();
    payload["op"] = std::string(op);
    payload["start"] = static_cast<Json::UInt64>(start);
    payload["end"] = static_cast<Json::UInt64>(end);
    payload["text"] = std::move(text);
    return payload;
  }

  Json::Value region_payload(const OverlayBox& box) const {
    Json::Value payload = context_payload();
    payload["region"] = box_json(box);
    return payload;
  }

  Json::Value region_remove_payload(const std::string& region_id) const {
    Json::Value payload = context_payload();
    payload["region_id"] = region_id;
    return payload;
  }

  Json::Value span_payload(const TextRegionSpan& span) const {
    Json::Value payload = context_payload();
    payload["span"] = span_json(span);
    return payload;
  }

  Json::Value metrics_payload(std::string_view status, std::string_view error) const {
    Json::Value payload = context_payload();
    payload["status"] = std::string(status);
    payload["error"] = error.empty() ? Json::Value(Json::nullValue) : Json::Value(std::string(error));
    payload["token_count"] = static_cast<Json::UInt64>(token_count_);
    payload["chunk_count"] = static_cast<Json::UInt64>(chunk_count_);
    payload["first_token_latency_ms"] = static_cast<Json::UInt64>(first_token_latency_ms_);
    payload["generation_duration_ms"] = static_cast<Json::UInt64>(generation_duration_ms_);
    payload["elapsed_ms"] = static_cast<Json::UInt64>(elapsed_ms_);
    payload["min_tps"] = min_tps_.has_value() ? *min_tps_ : 0.0;
    payload["max_tps"] = max_tps_;
    payload["avg_tps"] = avg_tps_;
    payload["started_at"] = started_at_;
    payload["first_token_at"] = first_token_at_;
    payload["completed_at"] = completed_at_;
    return payload;
  }

  void update_rates(std::chrono::steady_clock::time_point now) {
    elapsed_ms_ = elapsed_ms_since(started_, now);
    if (first_token_.has_value()) {
      generation_duration_ms_ = std::max<std::uint64_t>(1, elapsed_ms_since(*first_token_, now));
      avg_tps_ = static_cast<double>(token_count_) / (static_cast<double>(generation_duration_ms_) / 1000.0);
      if (avg_tps_ > 0) {
        min_tps_ = min_tps_.has_value() ? std::min(*min_tps_, avg_tps_) : avg_tps_;
        max_tps_ = std::max(max_tps_, avg_tps_);
      }
    }
  }

  void parse_and_publish() {
    const auto parsed = parse_ocr_markers(raw_text_,
                                          {.file_hash = file_hash_,
                                           .page_no = page_no_,
                                           .engine_id = "unlimited-ocr",
                                           .profile_id = profile_id_});
    WorkbenchService::Impl::PageState live_page;
    live_page.page_no = page_no_;
    live_page.raw_text = raw_text_;
    live_page.cleaned_text = parsed.cleaned_text.empty() ? raw_text_ : parsed.cleaned_text;
    live_page.boxes = to_overlay_boxes(parsed, page_no_);
    live_page.spans = parsed.text_region_spans;
    service_.apply_region_content(live_page);

    publish_text_delta(live_page.cleaned_text);
    publish_region_deltas(live_page.boxes);
    publish_span_deltas(live_page.spans);

    {
      std::scoped_lock lock(service_.mutex);
      auto found = service_.documents.find(file_hash_);
      if (found == service_.documents.end() || page_index_ >= found->second.pages.size()) {
        return;
      }
      auto& document = found->second;
      auto& page_state = document.pages[page_index_];
      page_state.raw_text = live_page.raw_text;
      page_state.cleaned_text = live_page.cleaned_text;
      page_state.boxes = live_page.boxes;
      page_state.spans = live_page.spans;
      service_.apply_region_content(page_state);
      service_.refresh_document_aggregate(document);
    }
  }

  void publish_text_delta(const std::string& next_text) {
    if (next_text == cleaned_text_) {
      return;
    }
    if (next_text.starts_with(cleaned_text_)) {
      service_.publish_event("ocr.page.text.patch",
                             text_patch_payload("append", cleaned_text_.size(), cleaned_text_.size(),
                                                next_text.substr(cleaned_text_.size())));
    } else {
      service_.publish_event("ocr.page.text.patch", text_patch_payload("replace", 0, cleaned_text_.size(), next_text));
    }
    cleaned_text_ = next_text;
  }

  void publish_region_deltas(const std::vector<OverlayBox>& boxes) {
    std::map<std::string, OverlayBox> next;
    for (const auto& box : boxes) {
      next[box.region_id] = box;
      const auto found = boxes_.find(box.region_id);
      if (found == boxes_.end() || !same_box(found->second, box)) {
        service_.publish_event("ocr.page.region.upsert", region_payload(box));
      }
    }
    for (const auto& [region_id, _] : boxes_) {
      if (!next.contains(region_id)) {
        service_.publish_event("ocr.page.region.remove", region_remove_payload(region_id));
      }
    }
    boxes_ = std::move(next);
  }

  void publish_span_deltas(const std::vector<TextRegionSpan>& spans) {
    std::map<std::string, TextRegionSpan> next;
    for (const auto& span : spans) {
      next[span.region_id] = span;
      const auto found = spans_.find(span.region_id);
      if (found == spans_.end() || !same_span(found->second, span)) {
        service_.publish_event("ocr.page.span.upsert", span_payload(span));
      }
    }
    for (const auto& [region_id, _] : spans_) {
      if (!next.contains(region_id)) {
        service_.publish_event("ocr.page.span.remove", region_remove_payload(region_id));
      }
    }
    spans_ = std::move(next);
  }

  WorkbenchService::Impl& service_;
  std::string run_id_;
  std::string file_hash_;
  std::size_t page_index_ = 0;
  int page_no_ = 1;
  std::string profile_id_;
  std::string model_id_;
  RuntimeVariant runtime_;
  std::chrono::steady_clock::time_point started_;
  std::optional<std::chrono::steady_clock::time_point> first_token_;
  std::string started_at_;
  std::string first_token_at_;
  std::string completed_at_;
  std::string raw_text_;
  std::string cleaned_text_;
  std::map<std::string, OverlayBox> boxes_;
  std::map<std::string, TextRegionSpan> spans_;
  std::uint64_t token_count_ = 0;
  std::uint64_t chunk_count_ = 0;
  std::uint64_t first_token_latency_ms_ = 0;
  std::uint64_t generation_duration_ms_ = 0;
  std::uint64_t elapsed_ms_ = 0;
  std::optional<double> min_tps_;
  double max_tps_ = 0.0;
  double avg_tps_ = 0.0;
};

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
  const auto runtime = selected_runtime();
  if (!runtime.selectable) {
    fail_run(run_id, "runtime is not ready: " + runtime.runtime_id + " (" + runtime.support_detail + ")");
    log_error(logger, "ingest", "run " + run_id + " failed because runtime is not selectable: " +
                                     runtime.runtime_id);
    return;
  }
  if (!std::filesystem::exists(runtime.ffi_library)) {
    fail_run(run_id, "uocr-ffi runtime is missing: " + runtime.ffi_library.string());
    log_error(logger, "ingest", "run " + run_id + " failed because uocr-ffi is missing");
    return;
  }
  log_info(logger, "models", "loading Unlimited-OCR runtime=" + runtime.runtime_id +
                                " accelerator=" + runtime.accelerator + " model=" + model_id +
                                " file=" + std::string(model_entry->model_file));
  UnlimitedOcrFfiEngine engine({runtime.ffi_library, model_path(model_id), mmproj_path(), runtime.n_gpu_layers},
                               *profile);
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
      PageStreamPublisher stream(*this, run_id, hash, index, page.page_no, profile->key, model_id, runtime);
      stream.start();
      OcrResult result;
      std::string error;
      try {
        result = engine.recognize_image({page.image_path, "document parsing.", profile->default_max_tokens},
                                        [&stream](const OcrEvent& event) {
                                          stream.on_event(event);
                                        });
        if (!result.ok) {
          error = result.error.empty() ? "OCR failed" : result.error;
        }
      } catch (const std::exception& exception) {
        error = exception.what();
      }
      const auto page_status = error.empty() ? std::string_view("completed") : std::string_view("failed");
      const auto page_metrics = stream.finish_metrics(page_status, error);

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
        persist_page_metrics(page_metrics);
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
      stream.publish_terminal(page_status, error);
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
