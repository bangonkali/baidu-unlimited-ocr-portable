#pragma once

#include <filesystem>

namespace uocr {

struct PngDimensions {
  int width_px = 0;
  int height_px = 0;
};

PngDimensions read_png_dimensions(const std::filesystem::path& path);

}  // namespace uocr
