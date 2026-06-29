#pragma once

#include <cstdint>
#include <functional>
#include <mutex>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include <json/json.h>

namespace uocr::server {

class RealtimeEventHub {
 public:
  using Subscriber = std::function<void(const std::string&)>;

  static RealtimeEventHub& instance();

  std::uint64_t subscribe(Subscriber subscriber);
  void unsubscribe(std::uint64_t subscriber_id);
  Json::Value publish(std::string_view type, const Json::Value& payload);
  std::string serialize_event(std::string_view type, const Json::Value& payload);
  std::size_t subscriber_count() const;

 private:
  Json::Value make_event(std::string_view type, const Json::Value& payload);

  mutable std::mutex mutex_;
  std::uint64_t next_subscriber_id_ = 1;
  std::uint64_t next_sequence_ = 1;
  std::vector<std::pair<std::uint64_t, Subscriber>> subscribers_;
};

std::vector<std::string_view> realtime_event_types();

}  // namespace uocr::server
