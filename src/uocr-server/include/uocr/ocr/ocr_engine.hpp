#pragma once

#include <filesystem>
#include <functional>
#include <string>

#include "uocr/core/profiles.hpp"

namespace uocr {

struct OcrRequest {
  std::filesystem::path image_path;
  std::string prompt = "document parsing.";
  int max_tokens = 8192;
};

struct OcrEvent {
  enum class Kind { Token, Done, Error };
  Kind kind = Kind::Token;
  std::string text;
  std::string message;
};

struct OcrResult {
  bool ok = false;
  std::string text;
  std::string error;
  int status_code = 0;
  std::uint64_t run_count = 0;
};

class OcrEngine {
 public:
  virtual ~OcrEngine() = default;
  virtual std::string id() const = 0;
  virtual OcrResult recognize_image(const OcrRequest& request,
                                    const std::function<void(const OcrEvent&)>& event_sink) = 0;
};

}  // namespace uocr

