#pragma once

#include <filesystem>

namespace uocr::server {

void register_api_routes(const std::filesystem::path& app_root);

}  // namespace uocr::server

