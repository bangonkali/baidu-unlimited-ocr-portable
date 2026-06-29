#pragma once

#include <filesystem>
#include <memory>

namespace uocr::server {

class AppLogger;

void register_api_routes(const std::filesystem::path& app_root, std::shared_ptr<AppLogger> logger);

}  // namespace uocr::server
