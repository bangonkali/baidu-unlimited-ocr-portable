#include "workbench_ingest_helpers.hpp"

#include <sstream>

#include "uocr/app/app_logger.hpp"

namespace uocr::server {

void log_info(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->info(component, message);
  }
}

void log_error(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->error(component, message);
  }
}

std::string page_label(const DiscoveredFile& file, int page_no, int page_count) {
  std::ostringstream out;
  out << file.relative_path.generic_string() << " page " << page_no << "/" << page_count;
  return out.str();
}

}  // namespace uocr::server
