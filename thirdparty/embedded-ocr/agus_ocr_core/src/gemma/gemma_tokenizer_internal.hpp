#ifndef AGUS_OCR_GEMMA_TOKENIZER_INTERNAL_HPP_
#define AGUS_OCR_GEMMA_TOKENIZER_INTERNAL_HPP_

#include <cstdint>
#include <string>

namespace agus_ocr {

constexpr const char* kGemmaSpaceToken = "\xE2\x96\x81";

std::string GemmaTokenizerUtf8Codepoint(uint32_t codepoint);
std::string GemmaTokenizerByteToken(unsigned char value);
std::string GemmaTokenizerMergeKey(const std::string& left,
                                   const std::string& right);
bool GemmaTokenizerIsByteToken(const std::string& token, unsigned char* value);

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_TOKENIZER_INTERNAL_HPP_
