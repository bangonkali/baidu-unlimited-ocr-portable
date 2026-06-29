#pragma once

#include <string>

namespace uocr::download {

struct HfAuthToken {
  std::string token;
  std::string source;

  [[nodiscard]] bool available() const { return !token.empty(); }
};

HfAuthToken read_hf_auth_from_environment();

}  // namespace uocr::download
