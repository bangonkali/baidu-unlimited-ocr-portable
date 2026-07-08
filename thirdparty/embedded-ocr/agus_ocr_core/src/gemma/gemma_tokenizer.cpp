#include "gemma/gemma_tokenizer.hpp"

#include <algorithm>
#include <limits>
#include <utility>

#include "gemma/gemma_common.hpp"
#include "gemma/gemma_tokenizer_internal.hpp"

namespace agus_ocr {

GemmaTokenizer::GemmaTokenizer(const std::string& tokenizer_json_path) {
  LoadTokenizerJson(tokenizer_json_path);
  eos_token_ids_ = {1, 50, 106};
  std::sort(special_tokens_.begin(), special_tokens_.end(),
            [](const auto& left, const auto& right) {
              return left.first.size() > right.first.size();
            });
}

int64_t GemmaTokenizer::TokenId(const std::string& token) const {
  const auto found = vocab_.find(token);
  if (found == vocab_.end()) {
    return unknown_id_;
  }
  return found->second;
}

bool GemmaTokenizer::IsEos(int64_t token_id) const {
  return eos_token_ids_.find(token_id) != eos_token_ids_.end();
}

std::vector<std::string> GemmaTokenizer::SegmentToPieces(
    const std::string& segment) const {
  std::vector<std::string> pieces;
  for (size_t i = 0; i < segment.size();) {
    const unsigned char c = static_cast<unsigned char>(segment[i]);
    size_t length = 1;
    if ((c & 0xe0) == 0xc0) length = 2;
    else if ((c & 0xf0) == 0xe0) length = 3;
    else if ((c & 0xf8) == 0xf0) length = 4;
    if (i + length > segment.size()) {
      length = 1;
    }
    std::string piece = segment.substr(i, length);
    if (vocab_.find(piece) != vocab_.end()) {
      pieces.push_back(std::move(piece));
    } else {
      for (size_t b = 0; b < length; ++b) {
        pieces.push_back(
            GemmaTokenizerByteToken(static_cast<unsigned char>(segment[i + b])));
      }
    }
    i += length;
  }
  return pieces;
}

std::vector<int64_t> GemmaTokenizer::EncodeTextSegment(
    const std::string& segment) const {
  std::vector<std::string> pieces = SegmentToPieces(segment);
  if (pieces.empty()) {
    return {};
  }
  while (pieces.size() > 1) {
    int32_t best_rank = std::numeric_limits<int32_t>::max();
    size_t best_index = pieces.size();
    for (size_t i = 0; i + 1 < pieces.size(); ++i) {
      const auto found =
          merge_ranks_.find(GemmaTokenizerMergeKey(pieces[i], pieces[i + 1]));
      if (found != merge_ranks_.end() && found->second < best_rank) {
        best_rank = found->second;
        best_index = i;
      }
    }
    if (best_index == pieces.size()) {
      break;
    }
    std::vector<std::string> merged;
    merged.reserve(pieces.size() - 1);
    for (size_t i = 0; i < pieces.size();) {
      if (i + 1 < pieces.size() && i == best_index) {
        merged.push_back(pieces[i] + pieces[i + 1]);
        i += 2;
      } else {
        merged.push_back(std::move(pieces[i]));
        ++i;
      }
    }
    pieces = std::move(merged);
  }

  std::vector<int64_t> ids;
  ids.reserve(pieces.size());
  for (const std::string& piece : pieces) {
    ids.push_back(TokenId(piece));
  }
  return ids;
}

std::vector<int64_t> GemmaTokenizer::Encode(const std::string& text) const {
  std::vector<int64_t> ids;
  std::string normalized;
  normalized.reserve(text.size());
  for (char c : text) {
    if (c == ' ') {
      normalized.append(kGemmaSpaceToken);
    } else {
      normalized.push_back(c);
    }
  }

  for (size_t i = 0; i < normalized.size();) {
    bool matched_special = false;
    for (const auto& special : special_tokens_) {
      if (normalized.compare(i, special.first.size(), special.first) == 0) {
        ids.push_back(special.second);
        i += special.first.size();
        matched_special = true;
        break;
      }
    }
    if (matched_special) {
      continue;
    }

    const size_t start = i;
    const bool newline = normalized[i] == '\n';
    while (i < normalized.size()) {
      bool next_special = false;
      for (const auto& special : special_tokens_) {
        if (normalized.compare(i, special.first.size(), special.first) == 0) {
          next_special = true;
          break;
        }
      }
      if (next_special || ((normalized[i] == '\n') != newline)) {
        break;
      }
      ++i;
    }
    const std::vector<int64_t> segment_ids =
        EncodeTextSegment(normalized.substr(start, i - start));
    ids.insert(ids.end(), segment_ids.begin(), segment_ids.end());
  }
  return ids;
}

std::string GemmaTokenizer::Decode(const std::vector<int64_t>& token_ids,
                                   bool skip_special_tokens) const {
  std::string out;
  std::vector<unsigned char> pending_bytes;
  const auto flush_bytes = [&]() {
    if (!pending_bytes.empty()) {
      out.append(reinterpret_cast<const char*>(pending_bytes.data()),
                 pending_bytes.size());
      pending_bytes.clear();
    }
  };

  for (int64_t id : token_ids) {
    if (id < 0 || static_cast<size_t>(id) >= id_to_token_.size()) {
      continue;
    }
    if (skip_special_tokens &&
        special_token_ids_.find(id) != special_token_ids_.end()) {
      continue;
    }
    std::string token = id_to_token_[static_cast<size_t>(id)];
    unsigned char byte_value = 0;
    if (GemmaTokenizerIsByteToken(token, &byte_value)) {
      pending_bytes.push_back(byte_value);
      continue;
    }
    flush_bytes();
    size_t pos = 0;
    while ((pos = token.find(kGemmaSpaceToken, pos)) != std::string::npos) {
      token.replace(pos, std::char_traits<char>::length(kGemmaSpaceToken),
                    " ");
      ++pos;
    }
    out.append(token);
  }
  flush_bytes();
  return GemmaTrim(out);
}

}  // namespace agus_ocr
