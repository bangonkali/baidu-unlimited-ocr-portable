#pragma once

#include "workbench_state.hpp"

#include <chrono>
#include <cstdint>
#include <map>
#include <optional>
#include <string>
#include <string_view>

#include "uocr/ocr/ocr_engine.hpp"

namespace uocr::server {

class PageStreamPublisher {
 public:
  PageStreamPublisher(WorkbenchService::Impl& service,
                      std::string run_id,
                      std::string file_hash,
                      std::size_t page_index,
                      int page_no,
                      std::string profile_id,
                      std::string model_id,
                      RuntimeVariant runtime);

  void start() const;
  void on_event(const OcrEvent& event);
  storage::OcrPageMetrics finish_metrics(std::string_view status, std::string_view error);
  void publish_terminal(std::string_view status, std::string_view error) const;

 private:
  Json::Value context_payload() const;
  Json::Value raw_delta_payload(const OcrEvent& event, std::size_t raw_start, std::size_t raw_end) const;
  Json::Value text_patch_payload(std::string_view op, std::size_t start, std::size_t end, std::string text) const;
  Json::Value box_json(const OverlayBox& box) const;
  Json::Value region_payload(const OverlayBox& box) const;
  Json::Value region_remove_payload(const std::string& region_id) const;
  Json::Value span_json(const TextRegionSpan& span) const;
  Json::Value span_payload(const TextRegionSpan& span) const;
  Json::Value metrics_payload(std::string_view status, std::string_view error) const;
  void update_rates(std::chrono::steady_clock::time_point now);
  void parse_and_publish();
  void publish_text_delta(const std::string& next_text);
  void publish_region_deltas(const std::vector<OverlayBox>& boxes);
  void publish_span_deltas(const std::vector<TextRegionSpan>& spans);

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

}  // namespace uocr::server
