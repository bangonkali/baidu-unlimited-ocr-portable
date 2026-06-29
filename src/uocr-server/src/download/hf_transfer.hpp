#pragma once

#include "uocr/download/hf_downloader.hpp"

namespace uocr::download::detail {

struct PreparedFile {
  HfFileSpec spec;
  std::string url;
  bool send_auth = false;
  std::uint64_t size = 0;
  std::string sha256;
};

void initialize_curl_once();

PreparedFile prepare_file(const HfFileSpec& spec, const HfDownloadOptions& options);

std::uint64_t existing_size(const std::filesystem::path& path);

void validate_download(const PreparedFile& file);

void download_prepared_file(const PreparedFile& file,
                            const HfDownloadOptions& options,
                            std::uint64_t completed_before,
                            std::uint64_t overall_total,
                            const HfDownloadProgressCallback& progress);

}  // namespace uocr::download::detail
