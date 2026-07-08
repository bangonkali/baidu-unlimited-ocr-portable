#ifndef AGUS_OCR_GEMMA_COMMON_HPP_
#define AGUS_OCR_GEMMA_COMMON_HPP_

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

namespace agus_ocr {

std::string GemmaReadTextFile(const std::string& path);
std::string GemmaJsonEscape(const std::string& value);
std::string GemmaBase64Encode(const std::vector<unsigned char>& bytes);
std::string GemmaTrim(const std::string& value);
int64_t GemmaElapsedMs(std::chrono::steady_clock::time_point start,
                       std::chrono::steady_clock::time_point end);
void GemmaLogInfo(const std::string& message);

#if defined(_WIN32)
std::wstring GemmaUtf8ToWide(const std::string& value);
#endif

}  // namespace agus_ocr

#endif  // AGUS_OCR_GEMMA_COMMON_HPP_
