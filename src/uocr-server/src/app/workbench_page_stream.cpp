#include "workbench_page_stream.hpp"

#include <algorithm>
#include <cmath>
#include <ctime>
#include <iomanip>
#include <sstream>
#include <utility>

#include "uocr/core/ocr_parser.hpp"

namespace uocr::server {
namespace {

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

}  // namespace

PageStreamPublisher::PageStreamPublisher(WorkbenchService::Impl& service,
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

void PageStreamPublisher::start() const {
  Json::Value payload = context_payload();
  payload["started_at"] = started_at_;
  service_.publish_event("ocr.page.stream.started", payload);
}

void PageStreamPublisher::on_event(const OcrEvent& event) {
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

storage::OcrPageMetrics PageStreamPublisher::finish_metrics(std::string_view status, std::string_view error) {
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

void PageStreamPublisher::publish_terminal(std::string_view status, std::string_view error) const {
  service_.publish_event(error.empty() ? "ocr.page.stream.completed" : "ocr.page.stream.failed",
                         metrics_payload(status, error));
}

void PageStreamPublisher::update_rates(std::chrono::steady_clock::time_point now) {
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

void PageStreamPublisher::parse_and_publish() {
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

void PageStreamPublisher::publish_text_delta(const std::string& next_text) {
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

void PageStreamPublisher::publish_region_deltas(const std::vector<OverlayBox>& boxes) {
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

void PageStreamPublisher::publish_span_deltas(const std::vector<TextRegionSpan>& spans) {
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

}  // namespace uocr::server
