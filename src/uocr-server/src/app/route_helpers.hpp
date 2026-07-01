#pragma once

#include <drogon/drogon.h>

namespace uocr::server {

inline drogon::HttpResponsePtr json_response(const Json::Value& value,
                                             drogon::HttpStatusCode status = drogon::k200OK) {
  auto response = drogon::HttpResponse::newHttpJsonResponse(value);
  response->setStatusCode(status);
  return response;
}

inline Json::Value request_json_or_empty(const drogon::HttpRequestPtr& request) {
  const auto json = request->getJsonObject();
  return json != nullptr ? *json : Json::Value(Json::objectValue);
}

}  // namespace uocr::server
