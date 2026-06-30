#include "uocr/core/ocr_parser.hpp"

#include <algorithm>
#include <cstdint>
#include <iomanip>
#include <regex>
#include <sstream>

namespace uocr {
namespace {

struct BoxPoints {
  double x1;
  double y1;
  double x2;
  double y2;
};

struct MarkerSegment {
  std::size_t start;
  std::size_t end;
  std::string label;
  std::vector<BoxPoints> boxes;
};

std::string trim(std::string value) {
  const auto first = value.find_first_not_of(" \t\r\n");
  if (first == std::string::npos) {
    return "";
  }
  const auto last = value.find_last_not_of(" \t\r\n");
  return value.substr(first, last - first + 1);
}

double clamp_coordinate(double value) {
  return std::max(0.0, std::min(999.0, value));
}

std::vector<BoxPoints> parse_box_points(const std::string& raw) {
  static const std::regex box_pattern(
      R"(\[\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\])");
  std::vector<BoxPoints> boxes;
  for (std::sregex_iterator it(raw.begin(), raw.end(), box_pattern), end; it != end; ++it) {
    BoxPoints points{std::stod((*it)[1].str()), std::stod((*it)[2].str()), std::stod((*it)[3].str()),
                     std::stod((*it)[4].str())};
    if (points.x2 < points.x1) {
      std::swap(points.x1, points.x2);
    }
    if (points.y2 < points.y1) {
      std::swap(points.y1, points.y2);
    }
    boxes.push_back({clamp_coordinate(points.x1), clamp_coordinate(points.y1),
                     clamp_coordinate(points.x2), clamp_coordinate(points.y2)});
  }
  return boxes;
}

bool span_is_inside(std::size_t start, std::size_t end, const std::vector<MarkerSegment>& segments) {
  return std::any_of(segments.begin(), segments.end(), [&](const MarkerSegment& segment) {
    return start >= segment.start && end <= segment.end;
  });
}

std::string remove_marker_tokens(std::string value) {
  static const std::regex marker_pattern(R"(<\|/?(?:ref|det)\|>)");
  return std::regex_replace(value, marker_pattern, "");
}

std::uint64_t fnv1a(std::string_view value) {
  std::uint64_t hash = 14695981039346656037ULL;
  for (const unsigned char ch : value) {
    hash ^= ch;
    hash *= 1099511628211ULL;
  }
  return hash;
}

std::string region_id_for(const ParseContext& context, const MarkerSegment& segment, const BoxPoints& box) {
  std::ostringstream key;
  key << context.file_hash << '|' << context.page_no << '|' << context.engine_id << '|'
      << context.profile_id << '|' << segment.start << ':' << segment.end << '|' << segment.label << '|'
      << box.x1 << ',' << box.y1 << ',' << box.x2 << ',' << box.y2;

  std::ostringstream out;
  out << "reg_" << std::hex << std::setw(16) << std::setfill('0') << fnv1a(key.str());
  return out.str();
}

void append_segment_text(std::string& cleaned, const std::string& text) {
  if (text.empty()) {
    return;
  }
  cleaned += text;
}

std::vector<MarkerSegment> collect_segments(const std::string& raw) {
  static const std::regex ref_pattern(
      R"(<\|ref\|>([\s\S]*?)<\|/ref\|>\s*<\|det\|>\s*([\s\S]*?)\s*<\|/det\|>)");
  static const std::regex det_pattern(R"(<\|det\|>\s*([\s\S]*?)\s*<\|/det\|>)");

  std::vector<MarkerSegment> segments;
  for (std::sregex_iterator it(raw.begin(), raw.end(), ref_pattern), end; it != end; ++it) {
    const auto& match = *it;
    auto boxes = parse_box_points(match[2].str());
    if (!boxes.empty()) {
      segments.push_back({static_cast<std::size_t>(match.position()),
                          static_cast<std::size_t>(match.position() + match.length()),
                          trim(match[1].str()), std::move(boxes)});
    }
  }

  for (std::sregex_iterator it(raw.begin(), raw.end(), det_pattern), end; it != end; ++it) {
    const auto& match = *it;
    const auto start = static_cast<std::size_t>(match.position());
    const auto stop = static_cast<std::size_t>(match.position() + match.length());
    if (span_is_inside(start, stop, segments)) {
      continue;
    }
    const std::string content = trim(match[1].str());
    const auto bracket_at = content.find('[');
    if (bracket_at == std::string::npos) {
      continue;
    }
    auto boxes = parse_box_points(content.substr(bracket_at));
    if (!boxes.empty()) {
      std::string label = trim(content.substr(0, bracket_at));
      segments.push_back({start, stop, label.empty() ? "det" : label, std::move(boxes)});
    }
  }

  std::sort(segments.begin(), segments.end(), [](const MarkerSegment& left, const MarkerSegment& right) {
    return left.start < right.start;
  });
  return segments;
}

}  // namespace

ParsedOcrPage parse_ocr_markers(std::string_view raw_text, const ParseContext& context) {
  const std::string raw(raw_text);
  ParsedOcrPage page;
  page.raw_text = raw;
  const auto segments = collect_segments(raw);

  std::size_t cursor = 0;
  for (const auto& segment : segments) {
    if (segment.start > cursor) {
      append_segment_text(page.cleaned_text, remove_marker_tokens(raw.substr(cursor, segment.start - cursor)));
    }

    const std::size_t clean_start = page.cleaned_text.size();
    append_segment_text(page.cleaned_text, segment.label);
    const std::size_t clean_end = page.cleaned_text.size();

    for (const auto& box : segment.boxes) {
      NormalizedBox parsed_box{region_id_for(context, segment, box), segment.label, box.x1, box.y1, box.x2,
                               box.y2};
      page.text_region_spans.push_back({parsed_box.region_id, context.page_no, clean_start, clean_end});
      page.boxes.push_back(std::move(parsed_box));
    }
    cursor = std::max(cursor, segment.end);
  }

  if (cursor < raw.size()) {
    append_segment_text(page.cleaned_text, remove_marker_tokens(raw.substr(cursor)));
  }
  return page;
}

std::vector<OverlayBox> to_overlay_boxes(const ParsedOcrPage& page, int page_no) {
  std::vector<OverlayBox> overlays;
  overlays.reserve(page.boxes.size());
  for (const auto& box : page.boxes) {
    overlays.push_back({
        .region_id = box.region_id,
        .label = box.label,
        .content_markdown = box.label,
        .content_html = "",
        .page_no = page_no,
        .left_percent = box.x1 / 999.0 * 100.0,
        .top_percent = box.y1 / 999.0 * 100.0,
        .width_percent = (box.x2 - box.x1) / 999.0 * 100.0,
        .height_percent = (box.y2 - box.y1) / 999.0 * 100.0,
        .hidden = false,
    });
  }
  return overlays;
}

}  // namespace uocr
