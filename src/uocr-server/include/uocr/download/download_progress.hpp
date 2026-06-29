#pragma once

#include <chrono>
#include <cstdint>
#include <optional>

namespace uocr::download {

double percent_complete(std::uint64_t downloaded, std::uint64_t total);

double transfer_rate_bytes_per_second(std::uint64_t transferred,
                                      std::chrono::steady_clock::duration elapsed);

std::optional<double> eta_seconds(std::uint64_t downloaded,
                                  std::uint64_t total,
                                  double bytes_per_second);

}  // namespace uocr::download
