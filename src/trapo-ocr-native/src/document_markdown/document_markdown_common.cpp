#include "document_markdown/document_markdown_common.hpp"

#include <algorithm>
#include <cctype>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <sstream>
#include <stdexcept>

#if defined(__ANDROID__)
#include <android/log.h>
#endif

#if defined(_WIN32)
#include <windows.h>
#endif

namespace trapo_ocr {
namespace {

bool IsUtf8Continuation(unsigned char value) {
  return (value & 0xc0) == 0x80;
}

void AppendUnicodeEscape(std::ostringstream* out, uint32_t codepoint) {
  const auto append_unit = [&](uint32_t unit) {
    *out << "\\u" << std::hex << std::setw(4) << std::setfill('0') << unit
         << std::dec << std::setfill(' ');
  };
  if (codepoint <= 0xffff) {
    append_unit(codepoint);
    return;
  }
  codepoint -= 0x10000;
  append_unit(0xd800 + (codepoint >> 10));
  append_unit(0xdc00 + (codepoint & 0x3ff));
}

uint32_t DecodeUtf8OrReplacement(const std::string& value, size_t* index) {
  const unsigned char first = static_cast<unsigned char>(value[*index]);
  if (first < 0x80) {
    ++(*index);
    return first;
  }

  uint32_t codepoint = 0;
  size_t length = 0;
  uint32_t minimum = 0;
  if ((first & 0xe0) == 0xc0) {
    codepoint = first & 0x1f;
    length = 2;
    minimum = 0x80;
  } else if ((first & 0xf0) == 0xe0) {
    codepoint = first & 0x0f;
    length = 3;
    minimum = 0x800;
  } else if ((first & 0xf8) == 0xf0) {
    codepoint = first & 0x07;
    length = 4;
    minimum = 0x10000;
  } else {
    ++(*index);
    return 0xfffd;
  }

  if (*index + length > value.size()) {
    ++(*index);
    return 0xfffd;
  }
  for (size_t offset = 1; offset < length; ++offset) {
    const unsigned char next =
        static_cast<unsigned char>(value[*index + offset]);
    if (!IsUtf8Continuation(next)) {
      ++(*index);
      return 0xfffd;
    }
    codepoint = (codepoint << 6) | (next & 0x3f);
  }
  if (codepoint < minimum || codepoint > 0x10ffff ||
      (codepoint >= 0xd800 && codepoint <= 0xdfff)) {
    ++(*index);
    return 0xfffd;
  }
  *index += length;
  return codepoint;
}

}  // namespace

std::string DocumentMarkdownReadTextFile(const std::string& path) {
  std::ifstream file(path, std::ios::binary);
  if (!file) {
    throw std::runtime_error("failed to open file: " + path);
  }
  std::ostringstream out;
  out << file.rdbuf();
  return out.str();
}

std::string DocumentMarkdownJsonEscape(const std::string& value) {
  std::ostringstream out;
  for (size_t i = 0; i < value.size();) {
    const uint32_t codepoint = DecodeUtf8OrReplacement(value, &i);
    switch (codepoint) {
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
        if (codepoint < 0x20 || codepoint >= 0x7f) {
          AppendUnicodeEscape(&out, codepoint);
        } else {
          out << static_cast<char>(codepoint);
        }
        break;
    }
  }
  return out.str();
}

std::string DocumentMarkdownBase64Encode(const std::vector<unsigned char>& bytes) {
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

std::string DocumentMarkdownTrim(const std::string& value) {
  const auto begin = value.find_first_not_of(" \t\r\n");
  if (begin == std::string::npos) {
    return "";
  }
  const auto end = value.find_last_not_of(" \t\r\n");
  return value.substr(begin, end - begin + 1);
}

int64_t DocumentMarkdownElapsedMs(std::chrono::steady_clock::time_point start,
                       std::chrono::steady_clock::time_point end) {
  return std::chrono::duration_cast<std::chrono::milliseconds>(end - start)
      .count();
}

void DocumentMarkdownLogInfo(const std::string& message) {
#if defined(__ANDROID__)
  __android_log_print(ANDROID_LOG_INFO, "TrapoOCR", "%s", message.c_str());
#else
  (void)message;
#endif
}

#if defined(_WIN32)
std::wstring DocumentMarkdownUtf8ToWide(const std::string& value) {
  if (value.empty()) {
    return std::wstring();
  }
  const int required = MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1,
                                           nullptr, 0);
  if (required <= 0) {
    throw std::runtime_error("failed to convert UTF-8 path to UTF-16");
  }
  std::wstring out(static_cast<size_t>(required - 1), L'\0');
  MultiByteToWideChar(CP_UTF8, 0, value.c_str(), -1, out.data(), required);
  return out;
}
#endif

}  // namespace trapo_ocr
