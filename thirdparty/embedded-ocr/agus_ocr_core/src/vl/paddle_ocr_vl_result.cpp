#include "vl/paddle_ocr_vl_result.hpp"

#include <algorithm>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <stdexcept>

#if defined(AGUS_OCR_USE_OPENCV_MOBILE) && \
    __has_include(<opencv2/highgui/highgui.hpp>)
#include <opencv2/highgui/highgui.hpp>
#else
#include <opencv2/imgcodecs.hpp>
#endif

namespace agus_ocr {
namespace {

std::string JsonEscape(const std::string& value) {
  std::ostringstream out;
  for (unsigned char c : value) {
    switch (c) {
      case '"':
        out << "\\\"";
        break;
      case '\\':
        out << "\\\\";
        break;
      case '\b':
        out << "\\b";
        break;
      case '\f':
        out << "\\f";
        break;
      case '\n':
        out << "\\n";
        break;
      case '\r':
        out << "\\r";
        break;
      case '\t':
        out << "\\t";
        break;
      default:
        if (c < 0x20) {
          out << "\\u" << std::hex << std::setw(4) << std::setfill('0')
              << static_cast<int>(c) << std::dec << std::setfill(' ');
        } else {
          out << static_cast<char>(c);
        }
        break;
    }
  }
  return out.str();
}

std::string Base64Encode(const std::vector<uchar>& bytes) {
  static constexpr char kAlphabet[] =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  std::string encoded;
  encoded.reserve(((bytes.size() + 2) / 3) * 4);
  size_t i = 0;
  while (i + 2 < bytes.size()) {
    const uint32_t value = (static_cast<uint32_t>(bytes[i]) << 16) |
                           (static_cast<uint32_t>(bytes[i + 1]) << 8) |
                           static_cast<uint32_t>(bytes[i + 2]);
    encoded.push_back(kAlphabet[(value >> 18) & 0x3f]);
    encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
    encoded.push_back(kAlphabet[(value >> 6) & 0x3f]);
    encoded.push_back(kAlphabet[value & 0x3f]);
    i += 3;
  }
  if (i < bytes.size()) {
    uint32_t value = static_cast<uint32_t>(bytes[i]) << 16;
    encoded.push_back(kAlphabet[(value >> 18) & 0x3f]);
    if (i + 1 < bytes.size()) {
      value |= static_cast<uint32_t>(bytes[i + 1]) << 8;
      encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
      encoded.push_back(kAlphabet[(value >> 6) & 0x3f]);
      encoded.push_back('=');
    } else {
      encoded.push_back(kAlphabet[(value >> 12) & 0x3f]);
      encoded.push_back('=');
      encoded.push_back('=');
    }
  }
  return encoded;
}

std::string EncodeImageBase64(const cv::Mat& page, std::string* mime_type) {
  std::vector<uchar> encoded;
  std::vector<int> params = {cv::IMWRITE_JPEG_QUALITY, 90};
  if (cv::imencode(".jpg", page, encoded, params)) {
    *mime_type = "image/jpeg";
    return Base64Encode(encoded);
  }
  if (cv::imencode(".png", page, encoded)) {
    *mime_type = "image/png";
    return Base64Encode(encoded);
  }
  throw std::runtime_error("failed to encode PaddleOCR-VL overlay image");
}

void AppendPointsJson(std::ostringstream* out,
                      const std::vector<cv::Point2f>& points) {
  *out << '[';
  for (size_t i = 0; i < points.size(); ++i) {
    if (i > 0) {
      *out << ',';
    }
    *out << "{\"x\":" << points[i].x << ",\"y\":" << points[i].y << "}";
  }
  *out << ']';
}

void AppendRectJson(std::ostringstream* out, const cv::Rect2f& box) {
  *out << "{\"left\":" << box.x << ",\"top\":" << box.y
       << ",\"right\":" << box.x + box.width
       << ",\"bottom\":" << box.y + box.height << "}";
}

std::string JoinBlocks(const std::vector<RecognizedBlock>& blocks,
                       bool markdown) {
  std::ostringstream out;
  bool first = true;
  for (const auto& block : blocks) {
    const std::string& value =
        markdown && !block.markdown.empty() ? block.markdown : block.text;
    if (value.empty()) {
      continue;
    }
    if (!first) {
      out << "\n\n";
    }
    first = false;
    out << value;
  }
  return out.str();
}

void AppendLinesJson(std::ostringstream* out,
                     const std::vector<RecognizedBlock>& blocks) {
  *out << '[';
  for (size_t i = 0; i < blocks.size(); ++i) {
    if (i > 0) {
      *out << ',';
    }
    const auto& block = blocks[i];
    *out << "{\"text\":\"" << JsonEscape(block.text)
         << "\",\"confidence\":" << block.region.confidence
         << ",\"polygon\":";
    AppendPointsJson(out, block.region.polygon);
    *out << ",\"boundingBox\":";
    AppendRectJson(out, block.region.bounding_box);
    *out << ",\"textLineAngle\":-1}";
  }
  *out << ']';
}

void AppendBlocksJson(std::ostringstream* out,
                      const std::vector<RecognizedBlock>& blocks) {
  *out << '[';
  for (size_t i = 0; i < blocks.size(); ++i) {
    if (i > 0) {
      *out << ',';
    }
    const auto& block = blocks[i];
    *out << "{\"id\":\"" << JsonEscape(block.id) << "\",\"label\":\""
         << JsonEscape(block.region.label) << "\",\"text\":\""
         << JsonEscape(block.text) << "\",\"markdown\":\""
         << JsonEscape(block.markdown)
         << "\",\"confidence\":" << block.region.confidence
         << ",\"readingOrder\":" << block.region.reading_order
         << ",\"polygon\":";
    AppendPointsJson(out, block.region.polygon);
    *out << ",\"boundingBox\":";
    AppendRectJson(out, block.region.bounding_box);
    *out << ",\"sourceLayerId\":\"source\"}";
  }
  *out << ']';
}

std::string StructuredJson(const std::vector<RecognizedBlock>& blocks) {
  std::ostringstream out;
  out << "{\"pipeline\":\"paddle_ocr_vl_1_6\",\"blocks\":";
  AppendBlocksJson(&out, blocks);
  out << '}';
  return out.str();
}

}  // namespace

cv::Mat DecodePaddleOcrVlImage(const agus_ocr_image_t& image) {
  std::vector<uint8_t> buffer(image.bytes, image.bytes + image.length);
  cv::Mat decoded = cv::imdecode(buffer, cv::IMREAD_COLOR);
  if (decoded.empty()) {
    throw std::invalid_argument("unsupported or invalid image bytes");
  }
  return decoded;
}

cv::Rect ClampPaddleOcrVlCrop(const cv::Rect2f& box, const cv::Size& size) {
  const int pad = 4;
  const int left = std::max(0, static_cast<int>(std::floor(box.x)) - pad);
  const int top = std::max(0, static_cast<int>(std::floor(box.y)) - pad);
  const int right = std::min(size.width,
                             static_cast<int>(std::ceil(box.x + box.width)) +
                                 pad);
  const int bottom = std::min(size.height,
                              static_cast<int>(std::ceil(box.y + box.height)) +
                                  pad);
  return cv::Rect(left, top, std::max(0, right - left),
                  std::max(0, bottom - top));
}

int64_t PaddleOcrVlElapsedMs(std::chrono::steady_clock::time_point start,
                             std::chrono::steady_clock::time_point end) {
  return std::chrono::duration_cast<std::chrono::milliseconds>(end - start)
      .count();
}

std::string BuildPaddleOcrVlResultJson(
    const cv::Mat& page,
    const std::vector<RecognizedBlock>& blocks,
    const PaddleOcrVlTiming& timing,
    const std::string& runtime_summary) {
  std::string overlay_mime;
  const std::string overlay_base64 = EncodeImageBase64(page, &overlay_mime);
  const std::string text = JoinBlocks(blocks, false);
  const std::string markdown = JoinBlocks(blocks, true);
  const std::string structured = StructuredJson(blocks);

  std::ostringstream out;
  out << "{\"status\":0,\"message\":\"ok\",\"pages\":[{\"pageIndex\":0,"
      << "\"width\":" << page.cols << ",\"height\":" << page.rows
      << ",\"overlayImageMimeType\":\"" << overlay_mime
      << "\",\"overlayImageBytesBase64\":\"" << overlay_base64
      << "\",\"docAngle\":-1,\"lines\":";
  AppendLinesJson(&out, blocks);
  out << ",\"annotationLayers\":[{\"id\":\"source\",\"label\":\"Original\","
      << "\"width\":" << page.cols << ",\"height\":" << page.rows
      << ",\"imageMimeType\":\"" << overlay_mime
      << "\",\"imageBytesBase64\":\"" << overlay_base64
      << "\",\"lines\":";
  AppendLinesJson(&out, blocks);
  out << ",\"geometry\":1,\"isAvailable\":true,\"confidence\":1.0,"
      << "\"message\":\"Exact PaddleOCR-VL block geometry in source image "
         "space.\"}]"
      << ",\"text\":\"" << JsonEscape(text) << "\",\"markdownText\":\""
      << JsonEscape(markdown) << "\",\"structuredJson\":\""
      << JsonEscape(structured) << "\",\"blocks\":";
  AppendBlocksJson(&out, blocks);
  out << "}],\"text\":\"" << JsonEscape(text) << "\",\"markdownText\":\""
      << JsonEscape(markdown) << "\",\"structuredJson\":\""
      << JsonEscape(structured)
      << "\",\"timing\":{\"docOrientationMs\":0,\"docUnwarpingMs\":0,"
      << "\"detectionMs\":" << timing.detection_ms
      << ",\"textLineOrientationMs\":0,\"recognitionMs\":"
      << timing.recognition_ms << ",\"totalMs\":" << timing.total_ms
      << "},\"modelSummary\":\"PaddleOCR-VL-1.6 native pipeline\","
      << "\"runtimeSummary\":\"" << JsonEscape(runtime_summary)
      << "\",\"warnings\":[]}";
  return out.str();
}

}  // namespace agus_ocr
