#include "routes.hpp"

#include <drogon/drogon.h>

#include <algorithm>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <memory>
#include <sstream>
#include <thread>

#include "folder_dialog.hpp"
#include "uocr/app/app_logger.hpp"
#include "uocr/app/workbench_service.hpp"

namespace uocr::server {
namespace {

#ifndef UOCR_APP_VERSION
#define UOCR_APP_VERSION "0.0.0-dev"
#endif
#ifndef UOCR_GIT_SHA
#define UOCR_GIT_SHA "unknown"
#endif
#ifndef UOCR_GIT_TAG
#define UOCR_GIT_TAG UOCR_APP_VERSION
#endif

std::filesystem::path source_root() {
#ifdef UOCR_SERVER_SOURCE_DIR
  return std::filesystem::path(UOCR_SERVER_SOURCE_DIR);
#else
  return std::filesystem::current_path();
#endif
}

std::filesystem::path resolve_openapi_path(const std::filesystem::path& app_root) {
  const auto bundled = app_root / "openapi" / "uocr.openapi.json";
  return std::filesystem::exists(bundled) ? bundled : source_root() / "openapi" / "uocr.openapi.json";
}
std::string read_text_file(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  std::ostringstream buffer;
  buffer << input.rdbuf();
  return buffer.str();
}

drogon::HttpResponsePtr json_response(const Json::Value& value,
                                       drogon::HttpStatusCode status = drogon::k200OK) {
  auto response = drogon::HttpResponse::newHttpJsonResponse(value);
  response->setStatusCode(status);
  return response;
}
Json::Value request_json_or_empty(const drogon::HttpRequestPtr& request) {
  const auto json = request->getJsonObject();
  return json != nullptr ? *json : Json::Value(Json::objectValue);
}

std::string compact_json(const Json::Value& value) {
  Json::StreamWriterBuilder builder;
  builder["indentation"] = "";
  return Json::writeString(builder, value);
}
std::string sse_frame(std::string_view event_name, const Json::Value& value) {
  return "event: " + std::string(event_name) + "\ndata: " + compact_json(value) + "\n\n";
}

void register_run_routes(const std::shared_ptr<WorkbenchService>& service) {
  using namespace drogon;
  app().registerHandler("/api/ingest/metrics/recent", [service](const HttpRequestPtr& request,
                                                                 std::function<void(const HttpResponsePtr&)>&& callback) {
    std::size_t limit = 50;
    const auto raw_limit = request->getParameter("limit");
    if (!raw_limit.empty()) {
      try {
        limit = static_cast<std::size_t>(std::max(1, std::stoi(raw_limit)));
      } catch (const std::exception&) {
        limit = 50;
      }
    }
    callback(json_response(service->recent_metrics(limit)));
  });

  app().registerHandler("/api/ingest/runs", [service](const HttpRequestPtr&,
                                                       std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->list_runs()));
  });
  app().registerHandler("/api/ingest/runs/{1}", [service](const HttpRequestPtr&,
                                                           std::function<void(const HttpResponsePtr&)>&& callback,
                                                           const std::string& run_id) {
    callback(json_response(service->get_run(run_id)));
  });

  app().registerHandler("/api/ingest/runs/{1}/metrics", [service](const HttpRequestPtr&,
                                                                   std::function<void(const HttpResponsePtr&)>&& callback,
                                                                   const std::string& run_id) {
    callback(json_response(service->run_metrics(run_id)));
  });

  app().registerHandler("/api/ingest/runs/{1}/stop",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& run_id) {
                          callback(json_response(service->run_command(run_id, "stop"), k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/ingest/runs/{1}/events", [service](const HttpRequestPtr&,
                                                                  std::function<void(const HttpResponsePtr&)>&& callback,
                                                                  const std::string& run_id) {
    auto response = HttpResponse::newHttpResponse();
    response->setStatusCode(k200OK);
    response->setContentTypeString("text/event-stream");
    response->setBody(service->run_event_stream(run_id));
    callback(response);
  });
}

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
                                    if (!service->model_downloading(model_id)) {
                                      ++terminal_frames;
                                    } else {
                                      terminal_frames = 0;
                                    }
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

void register_document_routes(const std::shared_ptr<WorkbenchService>& service) {
  using namespace drogon;
  app().registerHandler("/api/documents", [service](const HttpRequestPtr& request,
                                                     std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->list_documents(request->getParameter("q"))));
  });

  app().registerHandler("/api/search", [service](const HttpRequestPtr& request,
                                                  std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->list_documents(request->getParameter("q"))));
  });

  app().registerHandler("/api/documents/{1}", [service](const HttpRequestPtr&,
                                                         std::function<void(const HttpResponsePtr&)>&& callback,
                                                         const std::string& file_hash) {
    callback(json_response(service->get_document(file_hash)));
  });

  app().registerHandler("/api/documents/{1}/regions", [service](const HttpRequestPtr&,
                                                                 std::function<void(const HttpResponsePtr&)>&& callback,
                                                                 const std::string& file_hash) {
    callback(json_response(service->document_regions(file_hash)));
  });

  app().registerHandler("/api/documents/{1}/text", [service](const HttpRequestPtr&,
                                                              std::function<void(const HttpResponsePtr&)>&& callback,
                                                              const std::string& file_hash) {
    callback(json_response(service->document_text(file_hash)));
  });

  app().registerHandler("/api/documents/{1}/preview-images", [service](const HttpRequestPtr&,
                                                                        std::function<void(const HttpResponsePtr&)>&& callback,
                                                                        const std::string& file_hash) {
    callback(json_response(service->document_preview_images(file_hash)));
  });

  app().registerHandler("/api/documents/{1}/preview-images/{2}/{3}",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& file_hash, const std::string& variant, int page_no) {
                          const auto image = service->document_preview_image(file_hash, variant, page_no);
                          if (!image.has_value()) {
                            Json::Value payload;
                            payload["error"] = "preview image not found";
                            callback(json_response(payload, k404NotFound));
                            return;
                          }
                          callback(HttpResponse::newFileResponse(image->string()));
                        });

}

}  // namespace

void register_api_routes(const std::filesystem::path& app_root, std::shared_ptr<AppLogger> logger) {
  using namespace drogon;
  const auto openapi_path = resolve_openapi_path(app_root);
  auto service = std::make_shared<WorkbenchService>(app_root, logger);

  app().registerHandler("/api/health", [](const HttpRequestPtr&,
                                           std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["ok"] = true;
    payload["service"] = "uocr-server";
    callback(json_response(payload));
  });

  app().registerHandler("/api/status", [service](const HttpRequestPtr&,
                                                  std::function<void(const HttpResponsePtr&)>&& callback) {
    auto payload = service->status();
    payload["version"] = UOCR_APP_VERSION;
    payload["git_tag"] = UOCR_GIT_TAG;
    payload["git_sha"] = UOCR_GIT_SHA;
    callback(json_response(payload));
  });

  app().registerHandler("/api/openapi.json", [openapi_path](const HttpRequestPtr&,
                                                            std::function<void(const HttpResponsePtr&)>&& callback) {
    auto response = HttpResponse::newHttpResponse();
    response->setContentTypeCode(CT_APPLICATION_JSON);
    response->setBody(read_text_file(openapi_path));
    callback(response);
  }, {Get});

  register_model_routes(service);

  app().registerHandler("/api/settings", [service](const HttpRequestPtr&,
                                                    std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->settings()));
  }, {Get});

  app().registerHandler("/api/settings",
                        [service](const HttpRequestPtr& request, std::function<void(const HttpResponsePtr&)>&& callback) {
                          const auto payload = service->update_settings(request_json_or_empty(request));
                          callback(json_response(payload, payload.isMember("error") ? k400BadRequest : k200OK));
                        },
                        {Put});

  app().registerHandler("/api/system/folder-dialog", [logger](const HttpRequestPtr&,
                                                              std::function<void(const HttpResponsePtr&)>&& callback) {
    if (logger) {
      logger->info("folder", "folder picker requested");
    }
    callback(json_response(open_folder_dialog()));
  }, {Post});

  app().registerHandler("/api/ingest/start", [service](const HttpRequestPtr& request,
                                                        std::function<void(const HttpResponsePtr&)>&& callback) {
    try {
      callback(json_response(service->start_ingest(request_json_or_empty(request)), k202Accepted));
    } catch (const std::exception& error) {
      Json::Value payload;
      payload["error"] = error.what();
      callback(json_response(payload, k400BadRequest));
    }
  }, {Post});

  register_run_routes(service);
  register_document_routes(service);

  app().registerHandler("/api/logs/recent", [logger](const HttpRequestPtr& request,
                                                     std::function<void(const HttpResponsePtr&)>&& callback) {
    const auto raw_limit = request->getParameter("limit");
    int limit = 200;
    if (!raw_limit.empty()) {
      try {
        limit = std::stoi(raw_limit);
      } catch (const std::exception&) {
        limit = 200;
      }
    }
    callback(json_response(logger->recent_json(limit)));
  });
}

}  // namespace uocr::server
