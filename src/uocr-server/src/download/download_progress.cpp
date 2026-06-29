#include "uocr/download/download_progress.hpp"

#include <algorithm>

namespace uocr::download {

double percent_complete(std::uint64_t downloaded, std::uint64_t total) {
  if (total == 0) {
    return 0.0;
  }
  const auto clamped = std::min(downloaded, total);
  return (static_cast<double>(clamped) / static_cast<double>(total)) * 100.0;
}

double transfer_rate_bytes_per_second(std::uint64_t transferred,
                                      std::chrono::steady_clock::duration elapsed) {
  const auto seconds = std::chrono::duration<double>(elapsed).count();
  if (seconds <= 0.0) {
    return 0.0;
  }
  return static_cast<double>(transferred) / seconds;
}

std::optional<double> eta_seconds(std::uint64_t downloaded,
                                  std::uint64_t total,
                                  double bytes_per_second) {
  if (total == 0 || downloaded >= total || bytes_per_second <= 0.0) {
    return std::nullopt;
  }
  return static_cast<double>(total - downloaded) / bytes_per_second;
}

}  // namespace uocr::download
