#pragma once

#include <filesystem>
#include <string>

namespace uocr::download {

std::string sha256_file(const std::filesystem::path& path);

}  // namespace uocr::download
