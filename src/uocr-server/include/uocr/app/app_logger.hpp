#pragma once

#include <filesystem>
#include <mutex>
#include <string>
#include <string_view>

#include <json/json.h>

namespace uocr::server {

class AppLogger {
 public:
  explicit AppLogger(std::filesystem::path log_path);

  const std::filesystem::path& path() const;
  void info(std::string_view component, std::string_view message);
  void warn(std::string_view component, std::string_view message);
  void error(std::string_view component, std::string_view message);
  Json::Value recent_json(int limit) const;

 private:
  void write(std::string_view level, std::string_view component, std::string_view message);

  std::filesystem::path log_path_;
  mutable std::mutex mutex_;
};

}  // namespace uocr::server
