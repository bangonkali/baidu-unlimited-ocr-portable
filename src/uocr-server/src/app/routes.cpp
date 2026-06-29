#include "routes.hpp"

#include <drogon/drogon.h>

#include <filesystem>
#include <fstream>
#include <sstream>

#include "uocr/core/profiles.hpp"
#include "uocr/fs/file_scanner.hpp"

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
  if (std::filesystem::exists(bundled)) {
    return bundled;
  }
  return source_root() / "openapi" / "uocr.openapi.json";
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

Json::Value profile_json(const OcrProfileRecord& profile) {
  Json::Value value;
  value["key"] = profile.key;
  value["label"] = profile.label;
  value["engine_name"] = profile.engine_name;
  value["description"] = profile.description;
  value["default_max_tokens"] = profile.default_max_tokens;
  value["ngram_size"] = profile.ngram_size;
  value["ngram_window"] = profile.ngram_window;
  value["pdf_ngram_window"] = profile.pdf_ngram_window;
  value["force_prompt_eos"] = profile.force_prompt_eos;
  value["no_image_end"] = profile.no_image_end;
  return value;
}

void register_run_routes() {
  using namespace drogon;
  app().registerHandler("/api/ingest/runs", [](const HttpRequestPtr&,
                                                std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["runs"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/ingest/runs/{1}", [](const HttpRequestPtr&,
                                                    std::function<void(const HttpResponsePtr&)>&& callback,
                                                    const std::string& run_id) {
    Json::Value payload;
    payload["run_id"] = run_id;
    payload["root_path"] = "";
    payload["status"] = "unknown";
    callback(json_response(payload));
  });

  auto run_command = [](const HttpRequestPtr&, std::function<void(const HttpResponsePtr&)>&& callback,
                        const std::string& run_id) {
    Json::Value payload;
    payload["run_id"] = run_id;
    payload["root_path"] = "";
    payload["status"] = "requested";
    callback(json_response(payload, k202Accepted));
  };
  app().registerHandler("/api/ingest/runs/{1}/pause", run_command, {Post});
  app().registerHandler("/api/ingest/runs/{1}/resume", run_command, {Post});
  app().registerHandler("/api/ingest/runs/{1}/stop", run_command, {Post});
}

void register_document_routes() {
  using namespace drogon;
  app().registerHandler("/api/documents", [](const HttpRequestPtr&,
                                              std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["documents"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/documents/{1}", [](const HttpRequestPtr&,
                                                  std::function<void(const HttpResponsePtr&)>&& callback,
                                                  const std::string& file_hash) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["display_name"] = file_hash;
    payload["status"] = "unknown";
    payload["page_count"] = 0;
    callback(json_response(payload));
  });

  app().registerHandler("/api/documents/{1}/regions", [](const HttpRequestPtr&,
                                                          std::function<void(const HttpResponsePtr&)>&& callback,
                                                          const std::string& file_hash) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["boxes"] = Json::arrayValue;
    callback(json_response(payload));
  });

  app().registerHandler("/api/documents/{1}/text", [](const HttpRequestPtr&,
                                                       std::function<void(const HttpResponsePtr&)>&& callback,
                                                       const std::string& file_hash) {
    Json::Value payload;
    payload["file_hash"] = file_hash;
    payload["pages"] = Json::arrayValue;
    callback(json_response(payload));
  });
}

}  // namespace

void register_api_routes(const std::filesystem::path& app_root) {
  using namespace drogon;
  const auto openapi_path = resolve_openapi_path(app_root);

  app().registerHandler("/api/health", [](const HttpRequestPtr&,
                                           std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["ok"] = true;
    payload["service"] = "uocr-server";
    callback(json_response(payload));
  });

  app().registerHandler("/api/status", [](const HttpRequestPtr&,
                                           std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["state"] = "idle";
    payload["host"] = "127.0.0.1";
    payload["active_run_id"] = Json::nullValue;
    payload["default_profile"] = default_ocr_profile().key;
    payload["version"] = UOCR_APP_VERSION;
    payload["git_tag"] = UOCR_GIT_TAG;
    payload["git_sha"] = UOCR_GIT_SHA;
    for (const auto* suffix : {".pdf", ".png", ".jpg", ".jpeg", ".bmp", ".tif", ".tiff", ".webp"}) {
      payload["supported_inputs"].append(suffix);
    }
    callback(json_response(payload));
  });

  app().registerHandler("/api/openapi.json", [openapi_path](const HttpRequestPtr&,
                                                            std::function<void(const HttpResponsePtr&)>&& callback) {
    auto response = HttpResponse::newHttpResponse();
    response->setContentTypeCode(CT_APPLICATION_JSON);
    response->setBody(read_text_file(openapi_path));
    callback(response);
  }, {Get});

  app().registerHandler("/api/models", [](const HttpRequestPtr&,
                                           std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    Json::Value model;
    model["model_id"] = "unlimited-ocr-q4-k-m";
    model["display_name"] = "Unlimited-OCR Q4_K_M";
    model["status"] = "unknown";
    model["local_path"] = Json::nullValue;
    model["size_bytes"] = Json::nullValue;
    payload["models"].append(model);
    payload["profiles"] = Json::arrayValue;
    for (const auto& profile : ocr_profiles()) {
      payload["profiles"].append(profile_json(profile));
    }
    callback(json_response(payload));
  });

  app().registerHandler("/api/settings", [](const HttpRequestPtr&,
                                             std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["pdf_dpi"] = 200;
    payload["ocr_concurrency"] = 1;
    payload["default_profile"] = default_ocr_profile().key;
    payload["retry_profile"] = "experimental-exact-prefill-q4";
    callback(json_response(payload));
  }, {Get});

  app().registerHandler("/api/settings", [](const HttpRequestPtr& request,
                                             std::function<void(const HttpResponsePtr&)>&& callback) {
    const auto json = request->getJsonObject();
    callback(json_response(json != nullptr ? *json : Json::Value(Json::objectValue)));
  }, {Put});

  app().registerHandler("/api/system/folder-dialog", [](const HttpRequestPtr&,
                                                        std::function<void(const HttpResponsePtr&)>&& callback) {
    Json::Value payload;
    payload["cancelled"] = true;
    payload["selected_path"] = "";
    payload["manual_path_supported"] = true;
    callback(json_response(payload));
  }, {Post});

  app().registerHandler("/api/ingest/start", [](const HttpRequestPtr& request,
                                                 std::function<void(const HttpResponsePtr&)>&& callback) {
    const auto json = request->getJsonObject();
    const std::string root = json != nullptr ? (*json).get("root_path", "").asString() : "";
    try {
      const auto files = discover_supported_files(root);
      Json::Value payload;
      payload["run_id"] = "local-scaffold-run";
      payload["root_path"] = root;
      payload["status"] = "queued";
      payload["queued_files"] = static_cast<Json::UInt64>(files.size());
      callback(json_response(payload, k202Accepted));
    } catch (const std::exception& error) {
      Json::Value payload;
      payload["error"] = error.what();
      callback(json_response(payload, k400BadRequest));
    }
  }, {Post});

  register_run_routes();
  register_document_routes();

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
    const auto json = request->getJsonObject();
    callback(json_response(json != nullptr ? *json : Json::Value(Json::objectValue)));
  }, {Put});
}

}  // namespace uocr::server
