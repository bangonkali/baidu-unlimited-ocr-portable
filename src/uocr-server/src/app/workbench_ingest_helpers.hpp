#pragma once

#include <memory>
#include <string>
#include <string_view>

#include "uocr/fs/file_scanner.hpp"

namespace uocr::server {

class AppLogger;

void log_info(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message);
void log_error(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message);
std::string page_label(const DiscoveredFile& file, int page_no, int page_count);

}  // namespace uocr::server
