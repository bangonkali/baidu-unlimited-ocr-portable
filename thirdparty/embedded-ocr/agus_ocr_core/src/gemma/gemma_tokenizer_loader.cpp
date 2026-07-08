#include "gemma/gemma_tokenizer_json_scanner.hpp"

#include <stdexcept>

#include "gemma/gemma_common.hpp"
#include "gemma/gemma_tokenizer_internal.hpp"

namespace agus_ocr {

void TokenizerJsonScanner::Parse(GemmaTokenizer* tokenizer) {
  Expect('{');
  while (!Consume('}')) {
    const std::string key = ParseString();
    Expect(':');
    if (key == "model") {
      ParseModel(tokenizer);
    } else if (key == "added_tokens") {
      ParseAddedTokens(tokenizer);
    } else {
      SkipValue();
    }
    Consume(',');
  }
}

void TokenizerJsonScanner::ParseModel(GemmaTokenizer* tokenizer) {
  Expect('{');
  while (!Consume('}')) {
    const std::string key = ParseString();
    Expect(':');
    if (key == "vocab") {
      ParseVocab(tokenizer);
    } else if (key == "merges") {
      ParseMerges(tokenizer);
    } else if (key == "unk_token") {
      const std::string unk = ParseString();
      auto found = tokenizer->vocab_.find(unk);
      if (found != tokenizer->vocab_.end()) {
        tokenizer->unknown_id_ = found->second;
      }
    } else {
      SkipValue();
    }
    Consume(',');
  }
}

void TokenizerJsonScanner::ParseVocab(GemmaTokenizer* tokenizer) {
  Expect('{');
  while (!Consume('}')) {
    const std::string token = ParseString();
    Expect(':');
    const int64_t id = ParseInt();
    tokenizer->vocab_[token] = id;
    if (id >= 0) {
      if (static_cast<size_t>(id) >= tokenizer->id_to_token_.size()) {
        tokenizer->id_to_token_.resize(static_cast<size_t>(id) + 1);
      }
      tokenizer->id_to_token_[static_cast<size_t>(id)] = token;
    }
    Consume(',');
  }
}

void TokenizerJsonScanner::ParseMerges(GemmaTokenizer* tokenizer) {
  Expect('[');
  int32_t rank = 0;
  while (!Consume(']')) {
    Expect('[');
    const std::string left = ParseString();
    Expect(',');
    const std::string right = ParseString();
    Expect(']');
    tokenizer->merge_ranks_[GemmaTokenizerMergeKey(left, right)] = rank++;
    Consume(',');
  }
}

void TokenizerJsonScanner::ParseAddedTokens(GemmaTokenizer* tokenizer) {
  Expect('[');
  while (!Consume(']')) {
    Expect('{');
    int64_t id = -1;
    std::string content;
    bool special = false;
    while (!Consume('}')) {
      const std::string key = ParseString();
      Expect(':');
      if (key == "id") {
        id = ParseInt();
      } else if (key == "content") {
        content = ParseString();
      } else if (key == "special") {
        special = ParseBool();
      } else {
        SkipValue();
      }
      Consume(',');
    }
    if (id >= 0 && special && !content.empty()) {
      tokenizer->special_tokens_.push_back({content, id});
      tokenizer->special_token_ids_.insert(id);
    }
    Consume(',');
  }
}

void GemmaTokenizer::LoadTokenizerJson(const std::string& path) {
  const std::string json = GemmaReadTextFile(path);
  TokenizerJsonScanner(json).Parse(this);
  if (vocab_.empty() || merge_ranks_.empty() || id_to_token_.empty()) {
    throw std::runtime_error("Gemma tokenizer JSON did not contain vocab/merges");
  }
}

}  // namespace agus_ocr
