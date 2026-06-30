#include <algorithm>
#include <cassert>
#include <json/json.h>
#include <memory>
#include <stdexcept>
#include <string>

#include "uocr/app/realtime_event_hub.hpp"

namespace {

Json::Value parse_json(const std::string& text) {
  Json::CharReaderBuilder builder;
  Json::Value value;
  std::string errors;
  const auto reader = std::unique_ptr<Json::CharReader>(builder.newCharReader());
  const auto* begin = text.data();
  const auto* end = begin + text.size();
  assert(reader->parse(begin, end, &value, &errors));
  return value;
}

void test_publish_and_unsubscribe() {
  auto& hub = uocr::server::RealtimeEventHub::instance();
  int received = 0;
  std::string message;
  const auto id = hub.subscribe([&received, &message](const std::string& value) {
    ++received;
    message = value;
  });

  Json::Value payload;
  payload["model_id"] = "unlimited-ocr-q4-k-m";
  const auto event = hub.publish("model.changed", payload);

  assert(received == 1);
  assert(event["type"].asString() == "model.changed");
  assert(event["version"].asInt() == 1);
  assert(event["sequence"].asUInt64() > 0);

  const auto parsed = parse_json(message);
  assert(parsed["type"].asString() == "model.changed");
  assert(parsed["payload"]["model_id"].asString() == "unlimited-ocr-q4-k-m");

  hub.unsubscribe(id);
  hub.publish("model.changed", payload);
  assert(received == 1);
}

void test_supported_types_include_document_updates() {
  const auto types = uocr::server::realtime_event_types();
  const auto has_regions = std::find(types.begin(), types.end(), "document.regions.changed") != types.end();
  const auto has_logs = std::find(types.begin(), types.end(), "log.appended") != types.end();
  assert(has_regions);
  assert(has_logs);
}

void test_throwing_subscriber_does_not_escape_publish() {
  auto& hub = uocr::server::RealtimeEventHub::instance();
  const auto throwing_id = hub.subscribe([](const std::string&) {
    throw std::runtime_error("simulated disconnected client");
  });
  int received = 0;
  const auto healthy_id = hub.subscribe([&received](const std::string&) {
    ++received;
  });

  Json::Value payload;
  payload["status"] = "downloading";
  hub.publish("model.changed", payload);

  assert(received == 1);
  hub.unsubscribe(throwing_id);
  hub.unsubscribe(healthy_id);
}

}  // namespace

int main() {
  test_publish_and_unsubscribe();
  test_supported_types_include_document_updates();
  test_throwing_subscriber_does_not_escape_publish();
  return 0;
}
