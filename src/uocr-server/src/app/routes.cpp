#include "routes.hpp"

#include <drogon/drogon.h>

#include <filesystem>
#include <fstream>
#include <memory>
#include <sstream>

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

void register_run_routes(const std::shared_ptr<WorkbenchService>& service) {
  using namespace drogon;
  app().registerHandler("/api/ingest/runs", [service](const HttpRequestPtr&,
                                                       std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->list_runs()));
  });

  app().registerHandler("/api/ingest/runs/{1}", [service](const HttpRequestPtr&,
                                                           std::function<void(const HttpResponsePtr&)>&& callback,
                                                           const std::string& run_id) {
    callback(json_response(service->get_run(run_id)));
  });

  app().registerHandler("/api/ingest/runs/{1}/pause",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& run_id) {
                          callback(json_response(service->run_command(run_id, "pause"), k202Accepted));
                        },
                        {Post});
  app().registerHandler("/api/ingest/runs/{1}/resume",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& run_id) {
                          callback(json_response(service->run_command(run_id, "resume"), k202Accepted));
                        },
                        {Post});
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

void register_document_routes(const std::shared_ptr<WorkbenchService>& service) {
  using namespace drogon;
  app().registerHandler("/api/documents", [service](const HttpRequestPtr& request,
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

  app().registerHandler("/api/documents/{1}/preview-images", [](const HttpRequestPtr&,
                                                                 std::function<void(const HttpResponsePtr&)>&& callback,
                                                                 const std::string& file_hash) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["variants"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/documents/{1}/preview-images/{2}/{3}",
                        [](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                           const std::string&, const std::string&, int) {
                          Json::Value payload;
                          payload["error"] = "preview image not found";
                          callback(json_response(payload, k404NotFound));
                        });

  app().registerHandler("/api/documents/{1}/annotations/visibility",
                        [](const HttpRequestPtr& request, std::function<void(const HttpResponsePtr&)>&& callback,
                           const std::string& file_hash) {
                          auto payload = request_json_or_empty(request);
                          payload["file_hash"] = file_hash;
                          callback(json_response(payload));
                        },
                        {Put});
}

}  // namespace

void register_api_routes(const std::filesystem::path& app_root) {
  using namespace drogon;
  const auto openapi_path = resolve_openapi_path(app_root);
  auto service = std::make_shared<WorkbenchService>(app_root);

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

  app().registerHandler("/api/models", [service](const HttpRequestPtr&,
                                                  std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->models()));
  });

  app().registerHandler("/api/models/{1}/download",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          callback(json_response(service->start_model_download(model_id), k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/models/{1}/cancel",
                        [service](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                                  const std::string& model_id) {
                          callback(json_response(service->cancel_model_download(model_id), k202Accepted));
                        },
                        {Post});

  app().registerHandler("/api/settings", [service](const HttpRequestPtr&,
                                                    std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(service->settings()));
  }, {Get});

  app().registerHandler("/api/settings", [](const HttpRequestPtr& request,
                                             std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(request_json_or_empty(request)));
  }, {Put});

  app().registerHandler("/api/system/folder-dialog", [](const HttpRequestPtr&,
                                                        std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["cancelled"] = true;
    payload["selected_path"] = "";
    payload["manual_path_supported"] = true;
    callback(json_response(payload));
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

  app().registerHandler("/api/commands/search", [](const HttpRequestPtr&,
                                                   std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["commands"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/search", [](const HttpRequestPtr&,
                                           std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["results"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/annotation-settings", [](const HttpRequestPtr&,
                                                       std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["show_boxes"] = true;
    payload["show_labels"] = true;
    payload["box_color"] = "#4cc2ff";
    payload["active_box_color"] = "#e2b86b";
    callback(json_response(payload));
  }, {Get});

  app().registerHandler("/api/annotation-settings", [](const HttpRequestPtr& request,
                                                       std::function<void(const HttpResponsePtr&)>&& callback) {
    callback(json_response(request_json_or_empty(request)));
  }, {Put});
}

}  // namespace uocr::server
