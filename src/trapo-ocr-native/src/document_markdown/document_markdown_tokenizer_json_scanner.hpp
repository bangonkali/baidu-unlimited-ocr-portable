#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_JSON_SCANNER_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_JSON_SCANNER_HPP_

#include <cstddef>
#include <cstdint>
#include <string>
#include <utility>

#include "document_markdown/document_markdown_tokenizer.hpp"

namespace trapo_ocr {

class TokenizerJsonScanner {
 public:
  explicit TokenizerJsonScanner(const std::string& source);

  void Parse(DocumentMarkdownTokenizer* tokenizer);

 private:
  void ParseModel(DocumentMarkdownTokenizer* tokenizer);
  void ParseVocab(DocumentMarkdownTokenizer* tokenizer);
  void ParseMerges(DocumentMarkdownTokenizer* tokenizer);
  void ParseAddedTokens(DocumentMarkdownTokenizer* tokenizer);

  void SkipValue();
  std::string ParseString();
  std::string ParseUnicodeEscape();
  uint32_t ParseHex4();
  bool ParseBool();
  int64_t ParseInt();
  std::pair<const char*, size_t> ParseNumberSpan();
  bool Consume(char expected);
  void Expect(char expected);
  void ExpectRaw(char expected);
  bool Peek(char expected) const;
  void ConsumeLiteral(const char* literal);
  void SkipSpace();
  [[noreturn]] void Fail(const std::string& message) const;

  const std::string& source_;
  size_t offset_ = 0;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_JSON_SCANNER_HPP_
