#include "uocr/render/png_dimensions.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <fstream>
#include <stdexcept>

namespace uocr {
namespace {

int read_be_i32(const std::array<unsigned char, 24>& data, std::size_t offset) {
  const auto value = (static_cast<std::uint32_t>(data[offset]) << 24U) |
                     (static_cast<std::uint32_t>(data[offset + 1]) << 16U) |
                     (static_cast<std::uint32_t>(data[offset + 2]) << 8U) |
                     static_cast<std::uint32_t>(data[offset + 3]);
  return static_cast<int>(value);
}

}  // namespace

PngDimensions read_png_dimensions(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  if (!input) {
    throw std::runtime_error("PNG file cannot be opened: " + path.string());
  }

  std::array<unsigned char, 24> header{};
  input.read(reinterpret_cast<char*>(header.data()), static_cast<std::streamsize>(header.size()));
  if (input.gcount() != static_cast<std::streamsize>(header.size())) {
    throw std::runtime_error("PNG file is too small: " + path.string());
  }

  constexpr std::array<unsigned char, 8> signature = {0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n'};
  if (!std::equal(signature.begin(), signature.end(), header.begin())) {
    throw std::runtime_error("file is not a PNG image: " + path.string());
  }
  if (header[12] != 'I' || header[13] != 'H' || header[14] != 'D' || header[15] != 'R') {
    throw std::runtime_error("PNG IHDR chunk is missing: " + path.string());
  }

  return {.width_px = read_be_i32(header, 16), .height_px = read_be_i32(header, 20)};
}

}  // namespace uocr
