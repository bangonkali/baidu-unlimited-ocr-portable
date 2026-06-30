#pragma once

#include <cstddef>
#include <string>
#include <vector>

namespace uocr {

struct ParseContext {
  std::string file_hash = "unknown";
  int page_no = 1;
  std::string engine_id = "unlimited-ocr";
  std::string profile_id = "experimental-exact-prefill-q4";
};

struct NormalizedBox {
  std::string region_id;
  std::string label;
  double x1 = 0.0;
  double y1 = 0.0;
  double x2 = 0.0;
  double y2 = 0.0;
};

struct OverlayBox {
  std::string region_id;
  std::string label;
  std::string content_markdown;
  std::string content_html;
  int page_no = 1;
  double left_percent = 0.0;
  double top_percent = 0.0;
  double width_percent = 0.0;
  double height_percent = 0.0;
  bool hidden = false;
};

struct TextRegionSpan {
  std::string region_id;
  int page_no = 1;
  std::size_t start = 0;
  std::size_t end = 0;
};

struct ParsedOcrPage {
  std::string raw_text;
  std::string cleaned_text;
  std::vector<NormalizedBox> boxes;
  std::vector<TextRegionSpan> text_region_spans;
};

}  // namespace uocr
