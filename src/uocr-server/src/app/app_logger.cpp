#include "uocr/app/app_logger.hpp"

#include <algorithm>
#include <chrono>
#include <ctime>
#include <deque>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <utility>

namespace uocr::server {
namespace {

std::string utc_timestamp() {
  const auto now = std::chrono::system_clock::now();
  const auto time = std::chrono::system_clock::to_time_t(now);
  std::tm utc{};
#ifdef _WIN32
  gmtime_s(&utc, &time);
#else
  gmtime_r(&time, &utc);
#endif
  std::ostringstream stream;
  stream << std::put_time(&utc, "%Y-%m-%dT%H:%M:%SZ");
  return stream.str();
}

Json::Value parse_line(const std::string& line) {
  Json::Value item;
  const auto first = line.find(' ');
  const auto second = first == std::string::npos ? std::string::npos : line.find(' ', first + 1);
  const auto third = second == std::string::npos ? std::string::npos : line.find(' ', second + 1);
  if (first == std::string::npos || second == std::string::npos || third == std::string::npos) {
    item["timestamp"] = "";
    item["level"] = "INFO";
    item["component"] = "server";
    item["message"] = line;
    return item;
  }
  item["timestamp"] = line.substr(0, first);
  item["level"] = line.substr(first + 1, second - first - 1);
  item["component"] = line.substr(second + 1, third - second - 1);
  item["message"] = line.substr(third + 1);
  return item;
}

}  // namespace

AppLogger::AppLogger(std::filesystem::path log_path) : log_path_(std::move(log_path)) {}

const std::filesystem::path& AppLogger::path() const {
  return log_path_;
}

void AppLogger::set_sink(Sink sink) {
  std::scoped_lock lock(mutex_);
  sink_ = std::move(sink);
}

void AppLogger::info(std::string_view component, std::string_view message) {
  write("INFO", component, message);
}

void AppLogger::warn(std::string_view component, std::string_view message) {
  write("WARN", component, message);
}

void AppLogger::error(std::string_view component, std::string_view message) {
  write("ERROR", component, message);
}

void AppLogger::write(std::string_view level, std::string_view component, std::string_view message) {
  const auto line = utc_timestamp() + " " + std::string(level) + " " + std::string(component) +
                    " " + std::string(message);
  auto record = parse_line(line);
  Sink sink;
  {
    std::scoped_lock lock(mutex_);
    std::error_code error;
    std::filesystem::create_directories(log_path_.parent_path(), error);
    std::ofstream log(log_path_, std::ios::app);
    if (log) {
      log << line << '\n';
    }
    sink = sink_;
  }
  std::cout << line << std::endl;
  if (sink) {
    sink(record);
  }
}

Json::Value AppLogger::recent_json(int limit) const {
  const auto bounded_limit = std::clamp(limit, 1, 1000);
  std::deque<std::string> lines;
  {
    std::scoped_lock lock(mutex_);
    std::ifstream input(log_path_);
    std::string line;
    while (std::getline(input, line)) {
      lines.push_back(line);
      if (static_cast<int>(lines.size()) > bounded_limit) {
        lines.pop_front();
      }
    }
  }

  Json::Value payload;
  payload["log_path"] = log_path_.string();
  payload["logs"] = Json::arrayValue;
  for (const auto& line : lines) {
    payload["logs"].append(parse_line(line));
  }
  return payload;
}

}  // namespace uocr::server
