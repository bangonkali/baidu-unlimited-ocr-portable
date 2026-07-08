#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_INTERNAL_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_INTERNAL_HPP_

#include <cstdint>
#include <string>

namespace trapo_ocr {

constexpr const char* kDocumentMarkdownSpaceToken = "\xE2\x96\x81";

std::string DocumentMarkdownTokenizerUtf8Codepoint(uint32_t codepoint);
std::string DocumentMarkdownTokenizerByteToken(unsigned char value);
std::string DocumentMarkdownTokenizerMergeKey(const std::string& left,
                                   const std::string& right);
bool DocumentMarkdownTokenizerIsByteToken(const std::string& token, unsigned char* value);

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_INTERNAL_HPP_
