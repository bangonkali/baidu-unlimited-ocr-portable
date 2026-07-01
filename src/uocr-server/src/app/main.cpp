#include <drogon/drogon.h>

#include <chrono>
#include <ctime>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <filesystem>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <thread>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <shellapi.h>
#include <windows.h>
#endif

#include "uocr/app/app_logger.hpp"
#include "uocr/app/realtime_event_hub.hpp"
#include "routes.hpp"

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

bool has_flag(int argc, char* argv[], std::string_view flag) {
  for (int index = 1; index < argc; ++index) {
    if (std::string_view(argv[index]) == flag) {
      return true;
    }
  }
  return false;
}

void print_version() {
  std::cout << "uocr-server " << UOCR_APP_VERSION << '\n';
  std::cout << "git_tag " << UOCR_GIT_TAG << '\n';
  std::cout << "git_sha " << UOCR_GIT_SHA << '\n';
}

void print_help() {
  print_version();
  std::cout << "\nUsage: uocr-server [--port PORT] [--no-browser] [--version]\n";
}

void configure_process_dpi_awareness() {
#ifdef _WIN32
  if (SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)) {
    return;
  }
  SetProcessDPIAware();
#endif
}

std::filesystem::path executable_dir(const char* executable_path) {
  std::error_code error;
  auto path = std::filesystem::absolute(executable_path, error);
  if (error) {
    path = std::filesystem::current_path();
  }
  if (std::filesystem::is_regular_file(path, error)) {
    path = path.parent_path();
  }
  return std::filesystem::weakly_canonical(path, error);
}

std::uint16_t parse_port(int argc, char* argv[]) {
  int port = 8765;
  for (int index = 1; index < argc; ++index) {
    const std::string_view arg(argv[index]);
    if (arg == "--port" && index + 1 < argc) {
      port = std::stoi(argv[++index]);
      break;
    }
    if (!arg.empty() && arg.front() != '-') {
      port = std::stoi(std::string(arg));
      break;
    }
  }
  if (port < 1 || port > 65535) {
    throw std::out_of_range("port must be between 1 and 65535");
  }
  return static_cast<std::uint16_t>(port);
}

bool should_open_browser(int argc, char* argv[]) {
  for (int index = 1; index < argc; ++index) {
    if (std::string_view(argv[index]) == "--no-browser") {
      return false;
    }
  }
  return true;
}

void open_browser_after_start(std::uint16_t port) {
  std::thread([port]() {
    std::this_thread::sleep_for(std::chrono::milliseconds(600));
    const std::string url = "http://127.0.0.1:" + std::to_string(port) + "/";
#ifdef _WIN32
    ShellExecuteA(nullptr, "open", url.c_str(), nullptr, nullptr, SW_SHOWNORMAL);
#elif defined(__APPLE__)
    (void)std::system(("open '" + url + "' >/dev/null 2>&1").c_str());
#else
    (void)std::system(("xdg-open '" + url + "' >/dev/null 2>&1").c_str());
#endif
  }).detach();
}

int run_server(int argc, char* argv[]) {
  if (has_flag(argc, argv, "--version") || has_flag(argc, argv, "-v")) {
    print_version();
    return 0;
  }
  if (has_flag(argc, argv, "--help") || has_flag(argc, argv, "-h")) {
    print_help();
    return 0;
  }

  const auto app_root = executable_dir(argv[0]);
  const auto log_file = app_root / "logs" / "uocr-server.log";
  auto logger = std::make_shared<uocr::server::AppLogger>(log_file);
  logger->set_sink([](const Json::Value& record) {
    uocr::server::RealtimeEventHub::instance().publish("log.appended", record);
  });
  logger->info("server", "launch version=" UOCR_APP_VERSION " git_tag=" UOCR_GIT_TAG
                         " git_sha=" UOCR_GIT_SHA);

  const auto port = parse_port(argc, argv);
  const auto web_root = app_root / "web";
  const auto index_html = web_root / "index.html";
  if (!std::filesystem::exists(index_html)) {
    std::cerr << "Missing React build: " << index_html << '\n';
    std::cerr << "Run scripts\\windows\\build-workbench.ps1 or copy src\\uocr-client\\dist to web\\.\n";
    logger->error("server", "startup failed missing React build at " + index_html.string());
    return 2;
  }

  uocr::server::register_api_routes(app_root, logger);
  auto spa_response = drogon::HttpResponse::newFileResponse(index_html.string());
  spa_response->setStatusCode(drogon::k200OK);
  if (should_open_browser(argc, argv)) {
    open_browser_after_start(port);
  }

  logger->info("server", "app_root " + app_root.string());
  logger->info("server", "web_root " + web_root.string());
  logger->info("server", "log_path " + log_file.string());
  logger->info("server", "listening http://127.0.0.1:" + std::to_string(port) + "/");
  drogon::app()
      .setLogPath((app_root / "logs").string())
      .setLogLevel(trantor::Logger::kInfo)
      .setDocumentRoot(web_root.string())
      .setCustom404Page(spa_response, false)
      .addListener("127.0.0.1", port)
      .run();
  logger->info("server", "server stopped");
  return 0;
}

}  // namespace

int main(int argc, char* argv[]) {
  configure_process_dpi_awareness();
  try {
    return run_server(argc, argv);
  } catch (const std::exception& error) {
    const auto app_root = executable_dir(argc > 0 ? argv[0] : ".");
    uocr::server::AppLogger logger(app_root / "logs" / "uocr-server.log");
    logger.error("server", std::string("fatal startup error: ") + error.what());
    std::cerr << "uocr-server failed: " << error.what() << '\n';
    return 1;
  }
}
