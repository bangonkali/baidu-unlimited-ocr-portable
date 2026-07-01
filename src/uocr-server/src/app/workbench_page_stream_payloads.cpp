#include "workbench_page_stream.hpp"

#include <utility>

namespace uocr::server {

Json::Value PageStreamPublisher::context_payload() const {
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

Json::Value PageStreamPublisher::raw_delta_payload(const OcrEvent& event,
                                                   std::size_t raw_start,
                                                   std::size_t raw_end) const {
  Json::Value payload = context_payload();
  payload["token_index"] = static_cast<Json::UInt64>(event.index);
  payload["delta"] = event.text;
  payload["raw_start"] = static_cast<Json::UInt64>(raw_start);
  payload["raw_end"] = static_cast<Json::UInt64>(raw_end);
  payload["elapsed_ms"] = static_cast<Json::UInt64>(elapsed_ms_);
  payload["avg_tps"] = avg_tps_;
  return payload;
}

Json::Value PageStreamPublisher::text_patch_payload(std::string_view op,
                                                    std::size_t start,
                                                    std::size_t end,
                                                    std::string text) const {
  Json::Value payload = context_payload();
  payload["op"] = std::string(op);
  payload["start"] = static_cast<Json::UInt64>(start);
  payload["end"] = static_cast<Json::UInt64>(end);
  payload["text"] = std::move(text);
  return payload;
}

Json::Value PageStreamPublisher::box_json(const OverlayBox& box) const {
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

Json::Value PageStreamPublisher::region_payload(const OverlayBox& box) const {
  Json::Value payload = context_payload();
  payload["region"] = box_json(box);
  return payload;
}

Json::Value PageStreamPublisher::region_remove_payload(const std::string& region_id) const {
  Json::Value payload = context_payload();
  payload["region_id"] = region_id;
  return payload;
}

Json::Value PageStreamPublisher::span_json(const TextRegionSpan& span) const {
  Json::Value value;
  value["region_id"] = span.region_id;
  value["page_no"] = span.page_no;
  value["start"] = static_cast<Json::UInt64>(span.start);
  value["end"] = static_cast<Json::UInt64>(span.end);
  return value;
}

Json::Value PageStreamPublisher::span_payload(const TextRegionSpan& span) const {
  Json::Value payload = context_payload();
  payload["span"] = span_json(span);
  return payload;
}

Json::Value PageStreamPublisher::metrics_payload(std::string_view status, std::string_view error) const {
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

}  // namespace uocr::server
