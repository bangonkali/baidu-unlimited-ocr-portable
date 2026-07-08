#ifndef TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_HPP_
#define TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_HPP_

#include <cstdint>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace trapo_ocr {

class TokenizerJsonScanner;

class DocumentMarkdownTokenizer {
 public:
  explicit DocumentMarkdownTokenizer(const std::string& tokenizer_json_path);

  std::vector<int64_t> Encode(const std::string& text) const;
  std::string Decode(const std::vector<int64_t>& token_ids,
                     bool skip_special_tokens) const;

  int64_t TokenId(const std::string& token) const;
  bool IsEos(int64_t token_id) const;

 private:
  friend class TokenizerJsonScanner;

  std::vector<std::string> SegmentToPieces(const std::string& segment) const;
  std::vector<int64_t> EncodeTextSegment(const std::string& segment) const;
  void LoadTokenizerJson(const std::string& path);

  std::unordered_map<std::string, int64_t> vocab_;
  std::unordered_map<std::string, int32_t> merge_ranks_;
  std::vector<std::string> id_to_token_;
  std::vector<std::pair<std::string, int64_t>> special_tokens_;
  std::unordered_set<int64_t> special_token_ids_;
  std::unordered_set<int64_t> eos_token_ids_;
  int64_t unknown_id_ = 3;
};

}  // namespace trapo_ocr

#endif  // TRAPO_OCR_DOCUMENT_MARKDOWN_TOKENIZER_HPP_
