#ifndef AGUS_OCR_GEMMA_TOKENIZER_JSON_SCANNER_HPP_
#define AGUS_OCR_GEMMA_TOKENIZER_JSON_SCANNER_HPP_

#include <cstddef>
#include <cstdint>
#include <string>
#include <utility>

#include "gemma/gemma_tokenizer.hpp"

namespace agus_ocr {

class TokenizerJsonScanner {
 public:
  explicit TokenizerJsonScanner(const std::string& source);

  void Parse(GemmaTokenizer* tokenizer);

 private:
  void ParseModel(GemmaTokenizer* tokenizer);
  void ParseVocab(GemmaTokenizer* tokenizer);
  void ParseMerges(GemmaTokenizer* tokenizer);
  void ParseAddedTokens(GemmaTokenizer* tokenizer);

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

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_TOKENIZER_JSON_SCANNER_HPP_
