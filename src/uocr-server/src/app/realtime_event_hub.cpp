#include "uocr/app/realtime_event_hub.hpp"

#include <algorithm>
#include <chrono>
#include <ctime>
#include <exception>
#include <iomanip>
#include <sstream>
#include <utility>

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

std::string compact_json(const Json::Value& value) {
  Json::StreamWriterBuilder builder;
  builder["indentation"] = "";
  return Json::writeString(builder, value);
}

}  // namespace

RealtimeEventHub& RealtimeEventHub::instance() {
  static RealtimeEventHub hub;
  return hub;
}

std::uint64_t RealtimeEventHub::subscribe(Subscriber subscriber) {
  std::scoped_lock lock(mutex_);
  const auto id = next_subscriber_id_++;
  subscribers_.emplace_back(id, std::move(subscriber));
  return id;
}

void RealtimeEventHub::unsubscribe(std::uint64_t subscriber_id) {
  std::scoped_lock lock(mutex_);
  std::erase_if(subscribers_, [subscriber_id](const auto& item) {
    return item.first == subscriber_id;
  });
}

Json::Value RealtimeEventHub::publish(std::string_view type, const Json::Value& payload) {
  Json::Value event;
  std::vector<Subscriber> subscribers;
  {
    std::scoped_lock lock(mutex_);
    event = make_event(type, payload);
    subscribers.reserve(subscribers_.size());
    for (const auto& [_, subscriber] : subscribers_) {
      subscribers.push_back(subscriber);
    }
  }
  const auto message = compact_json(event);
  for (const auto& subscriber : subscribers) {
    try {
      subscriber(message);
    } catch (const std::exception&) {
    } catch (...) {
    }
  }
  return event;
}

std::string RealtimeEventHub::serialize_event(std::string_view type, const Json::Value& payload) {
  std::scoped_lock lock(mutex_);
  return compact_json(make_event(type, payload));
}

std::size_t RealtimeEventHub::subscriber_count() const {
  std::scoped_lock lock(mutex_);
  return subscribers_.size();
}

Json::Value RealtimeEventHub::make_event(std::string_view type, const Json::Value& payload) {
  Json::Value event;
  event["version"] = 1;
  event["sequence"] = static_cast<Json::UInt64>(next_sequence_++);
  event["type"] = std::string(type);
  event["occurred_at"] = utc_timestamp();
  event["payload"] = payload;
  return event;
}

std::vector<std::string_view> realtime_event_types() {
  return {
      "connection.ready",
      "status.changed",
      "model.changed",
      "run.changed",
      "document.changed",
      "document.page.changed",
      "document.regions.changed",
      "document.text.changed",
      "ocr.page.stream.started",
      "ocr.page.raw.delta",
      "ocr.page.text.patch",
      "ocr.page.region.upsert",
      "ocr.page.region.remove",
      "ocr.page.span.upsert",
      "ocr.page.span.remove",
      "ocr.page.metrics.changed",
      "ocr.page.stream.completed",
      "ocr.page.stream.failed",
      "log.appended",
  };
}

}  // namespace uocr::server
