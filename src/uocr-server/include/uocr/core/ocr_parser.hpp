#pragma once

#include <string>
#include <string_view>
#include <vector>

#include "uocr/core/types.hpp"

namespace uocr {

ParsedOcrPage parse_ocr_markers(std::string_view raw_text, const ParseContext& context);
std::vector<OverlayBox> to_overlay_boxes(const ParsedOcrPage& page, int page_no);

}  // namespace uocr

