#include <drogon/WebSocketController.h>

#include <cstdint>
#include <memory>
#include <string>

#include "uocr/app/realtime_event_hub.hpp"

namespace uocr::server {
namespace {

struct RealtimeSubscription {
  std::uint64_t id = 0;
};

Json::Value ready_payload() {
  Json::Value payload;
  payload["path"] = "/api/events";
  payload["heartbeat"] = "native-websocket";
  payload["supported_types"] = Json::arrayValue;
  for (const auto type : realtime_event_types()) {
    payload["supported_types"].append(std::string(type));
  }
  return payload;
}

}  // namespace

class WorkbenchRealtimeSocket : public drogon::WebSocketController<WorkbenchRealtimeSocket> {
 public:
  void handleNewMessage(const drogon::WebSocketConnectionPtr& connection,
                        std::string&&,
                        const drogon::WebSocketMessageType& type) override {
    if (type == drogon::WebSocketMessageType::Ping) {
      connection->send("", drogon::WebSocketMessageType::Pong);
    }
  }

  void handleNewConnection(const drogon::HttpRequestPtr&,
                           const drogon::WebSocketConnectionPtr& connection) override {
    connection->send(RealtimeEventHub::instance().serialize_event("connection.ready", ready_payload()));
    const std::weak_ptr<drogon::WebSocketConnection> weak_connection = connection;
    RealtimeSubscription subscription;
    subscription.id = RealtimeEventHub::instance().subscribe([weak_connection](const std::string& message) {
      if (const auto connection = weak_connection.lock()) {
        connection->send(message);
      }
    });
    connection->setContext(std::make_shared<RealtimeSubscription>(subscription));
  }

  void handleConnectionClosed(const drogon::WebSocketConnectionPtr& connection) override {
    const auto& subscription = connection->getContextRef<RealtimeSubscription>();
    RealtimeEventHub::instance().unsubscribe(subscription.id);
  }

  WS_PATH_LIST_BEGIN
  WS_PATH_ADD("/api/events", drogon::Get);
  WS_PATH_LIST_END
};

}  // namespace uocr::server
