#include "uocr/download/hf_auth.hpp"

#include <cstdlib>
#include <utility>

namespace uocr::download {
namespace {

std::string env_value(const char* name) {
#ifdef _WIN32
  char* value = nullptr;
  std::size_t size = 0;
  if (_dupenv_s(&value, &size, name) != 0 || value == nullptr) {
    return {};
  }
  std::string result(value);
  std::free(value);
  return result;
#else
  const char* value = std::getenv(name);
  return value == nullptr ? std::string() : std::string(value);
#endif
}

}  // namespace

HfAuthToken read_hf_auth_from_environment() {
  auto token = env_value("HF_TOKEN");
  if (!token.empty()) {
    return {.token = std::move(token), .source = "HF_TOKEN"};
  }
  token = env_value("HUGGING_FACE_HUB_TOKEN");
  if (!token.empty()) {
    return {.token = std::move(token), .source = "HUGGING_FACE_HUB_TOKEN"};
  }
  return {};
}

}  // namespace uocr::download
