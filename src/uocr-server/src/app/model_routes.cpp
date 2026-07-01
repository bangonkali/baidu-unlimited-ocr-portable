#include <drogon/drogon.h>

#include <chrono>
#include <memory>
#include <sstream>
#include <string_view>
#include <thread>
#include <utility>

#include "route_helpers.hpp"
#include "uocr/app/workbench_service.hpp"

namespace uocr::server {
namespace {

std::string compact_json(const Json::Value& value) {
  Json::StreamWriterBuilder builder;
  builder["indentation"] = "";
  return Json::writeString(builder, value);
}

std::string sse_frame(std::string_view event_name, const Json::Value& value) {
  return "event: " + std::string(event_name) + "\ndata: " + compact_json(value) + "\n\n";
}

}  // namespace

void register_model_routes(const std::shared_ptr<WorkbenchService>& service) {
  using namespace drogon;
  app().registerHandler("/api/models", [service](const HttpRequestPtr&,
                                                  std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->models()));
  });

  app().registerHandler("/api/models/{1}/download",
                        [service](const HttpRequestPtr& request, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          const auto body = request_json_or_empty(request);
                          callback(json_response(service->start_model_download(model_id, body.get("force", false).asBool()),
                                                 k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/models/{1}/select",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          callback(json_response(service->select_model(model_id), k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/models/{1}/cancel",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          callback(json_response(service->cancel_model_download(model_id), k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/models/{1}/events",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          auto response = HttpResponse::newAsyncStreamResponse(
                              [service, model_id](ResponseStreamPtr stream) {
                                std::thread([service, model_id,
                                             stream = std::shared_ptr<ResponseStream>(std::move(stream))]() {
                                  int terminal_frames = 0;
                                  std::string previous;
                                  while (terminal_frames < 3) {
                                    const auto event = service->model_download_event(model_id);
                                    const auto frame = sse_frame("model", event);
                                    if (frame != previous || terminal_frames == 0) {
                                      previous = frame;
                                      if (!stream->send(frame)) {
                                        stream->close();
                                        return;
                                      }
                                    }
                                    terminal_frames = service->model_downloading(model_id) ? 0 : terminal_frames + 1;
                                    std::this_thread::sleep_for(std::chrono::milliseconds(500));
                                  }
                                  stream->close();
                                }).detach();
                              },
                              true);
                          response->setContentTypeCodeAndCustomString(CT_CUSTOM, "text/event-stream");
                          response->addHeader("Cache-Control", "no-cache");
                          callback(response);
                        },
                        {Get});
}

}  // namespace uocr::server
