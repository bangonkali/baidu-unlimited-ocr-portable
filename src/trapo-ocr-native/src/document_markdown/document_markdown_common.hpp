#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_COMMON_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_COMMON_HPP_

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

namespace trapo_ocr {

std::string DocumentMarkdownReadTextFile(const std::string& path);
std::string DocumentMarkdownJsonEscape(const std::string& value);
std::string DocumentMarkdownBase64Encode(const std::vector<unsigned char>& bytes);
std::string DocumentMarkdownTrim(const std::string& value);
int64_t DocumentMarkdownElapsedMs(std::chrono::steady_clock::time_point start,
                       std::chrono::steady_clock::time_point end);
void DocumentMarkdownLogInfo(const std::string& message);

#if defined(_WIN32)
std::wstring DocumentMarkdownUtf8ToWide(const std::string& value);
#endif

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_COMMON_HPP_
