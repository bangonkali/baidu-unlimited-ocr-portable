#include "document_markdown/document_markdown_tokenizer_json_scanner.hpp"

#include <cctype>
#include <stdexcept>

#include "document_markdown/document_markdown_tokenizer_internal.hpp"

namespace trapo_ocr {

TokenizerJsonScanner::TokenizerJsonScanner(const std::string& source)
    : source_(source) {}

void TokenizerJsonScanner::SkipValue() {
  SkipSpace();
  if (Consume('{')) {
    while (!Consume('}')) {
      ParseString();
      Expect(':');
      SkipValue();
      Consume(',');
    }
    return;
  }
  if (Consume('[')) {
    while (!Consume(']')) {
      SkipValue();
      Consume(',');
    }
    return;
  }
  if (Peek('"')) {
    ParseString();
    return;
  }
  if (Peek('t') || Peek('f')) {
    ParseBool();
    return;
  }
  if (Peek('n')) {
    ConsumeLiteral("null");
    return;
  }
  ParseNumberSpan();
}

std::string TokenizerJsonScanner::ParseString() {
  SkipSpace();
  ExpectRaw('"');
  std::string out;
  while (offset_ < source_.size()) {
    const char c = source_[offset_++];
    if (c == '"') {
      return out;
    }
    if (c != '\\') {
      out.push_back(c);
      continue;
    }
    if (offset_ >= source_.size()) {
      Fail("unfinished JSON string escape");
    }
    const char escaped = source_[offset_++];
    switch (escaped) {
      case '"':
      case '\\':
      case '/':
        out.push_back(escaped);
        break;
      case 'b':
        out.push_back('\b');
        break;
      case 'f':
        out.push_back('\f');
        break;
      case 'n':
        out.push_back('\n');
        break;
      case 'r':
        out.push_back('\r');
        break;
      case 't':
        out.push_back('\t');
        break;
      case 'u':
        out.append(ParseUnicodeEscape());
        break;
      default:
        Fail("unsupported JSON string escape");
    }
  }
  Fail("unterminated JSON string");
}

std::string TokenizerJsonScanner::ParseUnicodeEscape() {
  uint32_t codepoint = ParseHex4();
  if (codepoint >= 0xd800 && codepoint <= 0xdbff &&
      offset_ + 6 <= source_.size() && source_[offset_] == '\\' &&
      source_[offset_ + 1] == 'u') {
    offset_ += 2;
    const uint32_t low = ParseHex4();
    if (low >= 0xdc00 && low <= 0xdfff) {
      codepoint = 0x10000 + ((codepoint - 0xd800) << 10) + (low - 0xdc00);
    }
  }
  return DocumentMarkdownTokenizerUtf8Codepoint(codepoint);
}

uint32_t TokenizerJsonScanner::ParseHex4() {
  if (offset_ + 4 > source_.size()) {
    Fail("short JSON unicode escape");
  }
  uint32_t value = 0;
  for (int i = 0; i < 4; ++i) {
    const char c = source_[offset_++];
    value <<= 4;
    if (c >= '0' && c <= '9') {
      value += c - '0';
    } else if (c >= 'a' && c <= 'f') {
      value += 10 + c - 'a';
    } else if (c >= 'A' && c <= 'F') {
      value += 10 + c - 'A';
    } else {
      Fail("invalid JSON unicode escape");
    }
  }
  return value;
}

bool TokenizerJsonScanner::ParseBool() {
  SkipSpace();
  if (source_.compare(offset_, 4, "true") == 0) {
    offset_ += 4;
    return true;
  }
  if (source_.compare(offset_, 5, "false") == 0) {
    offset_ += 5;
    return false;
  }
  Fail("invalid JSON boolean");
}

int64_t TokenizerJsonScanner::ParseInt() {
  const auto span = ParseNumberSpan();
  return std::stoll(std::string(span.first, span.second));
}

std::pair<const char*, size_t> TokenizerJsonScanner::ParseNumberSpan() {
  SkipSpace();
  const size_t start = offset_;
  if (Peek('-')) ++offset_;
  while (offset_ < source_.size() &&
         std::isdigit(static_cast<unsigned char>(source_[offset_]))) {
    ++offset_;
  }
  if (Peek('.')) {
    ++offset_;
    while (offset_ < source_.size() &&
           std::isdigit(static_cast<unsigned char>(source_[offset_]))) {
      ++offset_;
    }
  }
  if (Peek('e') || Peek('E')) {
    ++offset_;
    if (Peek('-') || Peek('+')) ++offset_;
    while (offset_ < source_.size() &&
           std::isdigit(static_cast<unsigned char>(source_[offset_]))) {
      ++offset_;
    }
  }
  if (offset_ == start) {
    Fail("expected JSON number");
  }
  return {source_.data() + start, offset_ - start};
}

bool TokenizerJsonScanner::Consume(char expected) {
  SkipSpace();
  if (!Peek(expected)) {
    return false;
  }
  ++offset_;
  return true;
}

void TokenizerJsonScanner::Expect(char expected) {
  SkipSpace();
  ExpectRaw(expected);
}

void TokenizerJsonScanner::ExpectRaw(char expected) {
  if (!Peek(expected)) {
    Fail("unexpected JSON character");
  }
  ++offset_;
}

bool TokenizerJsonScanner::Peek(char expected) const {
  return offset_ < source_.size() && source_[offset_] == expected;
}

void TokenizerJsonScanner::ConsumeLiteral(const char* literal) {
  const size_t length = std::char_traits<char>::length(literal);
  if (source_.compare(offset_, length, literal) != 0) {
    Fail("invalid JSON literal");
  }
  offset_ += length;
}

void TokenizerJsonScanner::SkipSpace() {
  while (offset_ < source_.size() &&
         std::isspace(static_cast<unsigned char>(source_[offset_]))) {
    ++offset_;
  }
}

void TokenizerJsonScanner::Fail(const std::string& message) const {
  throw std::runtime_error("DocumentMarkdown tokenizer JSON parse failed: " + message);
}

}  // namespace trapo_ocr
