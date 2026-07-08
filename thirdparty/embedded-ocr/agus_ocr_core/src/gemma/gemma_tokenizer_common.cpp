#include "gemma/gemma_tokenizer_internal.hpp"

#include <iomanip>
#include <sstream>

namespace agus_ocr {
namespace {

constexpr char kMergeSeparator = '\x1f';

}  // namespace

std::string GemmaTokenizerUtf8Codepoint(uint32_t codepoint) {
  std::string out;
  if (codepoint <= 0x7f) {
    out.push_back(static_cast<char>(codepoint));
  } else if (codepoint <= 0x7ff) {
    out.push_back(static_cast<char>(0xc0 | (codepoint >> 6)));
    out.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
  } else if (codepoint <= 0xffff) {
    out.push_back(static_cast<char>(0xe0 | (codepoint >> 12)));
    out.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3f)));
    out.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
  } else {
    out.push_back(static_cast<char>(0xf0 | (codepoint >> 18)));
    out.push_back(static_cast<char>(0x80 | ((codepoint >> 12) & 0x3f)));
    out.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3f)));
    out.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
  }
  return out;
}

std::string GemmaTokenizerByteToken(unsigned char value) {
  std::ostringstream out;
  out << "<0x" << std::uppercase << std::hex << std::setw(2)
      << std::setfill('0') << static_cast<int>(value) << ">";
  return out.str();
}

std::string GemmaTokenizerMergeKey(const std::string& left,
                                   const std::string& right) {
  std::string key;
  key.reserve(left.size() + right.size() + 1);
  key.append(left);
  key.push_back(kMergeSeparator);
  key.append(right);
  return key;
}

bool GemmaTokenizerIsByteToken(const std::string& token,
                               unsigned char* value) {
  if (token.size() != 6 || token[0] != '<' || token[1] != '0' ||
      token[2] != 'x' || token[5] != '>') {
    return false;
  }
  const auto hex = [](char c) -> int {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'A' && c <= 'F') return 10 + c - 'A';
    if (c >= 'a' && c <= 'f') return 10 + c - 'a';
    return -1;
  };
  const int hi = hex(token[3]);
  const int lo = hex(token[4]);
  if (hi < 0 || lo < 0) {
    return false;
  }
  *value = static_cast<unsigned char>((hi << 4) | lo);
  return true;
}

}  // namespace agus_ocr
