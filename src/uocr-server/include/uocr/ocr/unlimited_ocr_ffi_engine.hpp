#pragma once

#include <filesystem>
#include <memory>

#include "uocr/ocr/ocr_engine.hpp"

namespace uocr {

struct UnlimitedOcrRuntimePaths {
  std::filesystem::path ffi_library;
  std::filesystem::path model;
  std::filesystem::path mmproj;
};

class UnlimitedOcrFfiEngine final : public OcrEngine {
 public:
  UnlimitedOcrFfiEngine(UnlimitedOcrRuntimePaths paths, OcrProfileRecord profile);
  ~UnlimitedOcrFfiEngine() override;

  UnlimitedOcrFfiEngine(const UnlimitedOcrFfiEngine&) = delete;
  UnlimitedOcrFfiEngine& operator=(const UnlimitedOcrFfiEngine&) = delete;

  std::string id() const override;
  OcrResult recognize_image(const OcrRequest& request,
                            const std::function<void(const OcrEvent&)>& event_sink) override;

 private:
  struct Impl;
  std::unique_ptr<Impl> impl_;
};

}  // namespace uocr

