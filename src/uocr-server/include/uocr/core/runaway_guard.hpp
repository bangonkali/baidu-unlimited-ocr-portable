#pragma once

#include <optional>
#include <string>
#include <string_view>

namespace uocr {

std::optional<std::string> detect_recoverable_output_issue(std::string_view text);

}  // namespace uocr

