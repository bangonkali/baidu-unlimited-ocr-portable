#pragma once

#include <string>
#include <vector>

namespace uocr {

struct PromptProfileRecord {
  std::string key;
  std::string label;
  std::string prompt;
  std::string description;
};

struct OcrProfileRecord {
  std::string key;
  std::string label;
  std::string engine_name;
  std::string description;
  bool force_prompt_eos = true;
  bool no_image_end = false;
  std::string deepseek_ocr_mode = "gundam";
  std::string media_placement = "prefix-tight";
  bool no_repeat_ngram = true;
  int ngram_size = 35;
  int ngram_window = 128;
  int pdf_ngram_window = 1024;
  std::vector<int> ngram_whitelist = {128821, 128822};
  bool prefill_aware_swa = true;
  int decode_window = 128;
  int ctx_size = 32768;
  int default_max_tokens = 8192;
};

const std::vector<PromptProfileRecord>& prompt_profiles();
const std::vector<OcrProfileRecord>& ocr_profiles();
const OcrProfileRecord& default_ocr_profile();
const OcrProfileRecord* find_ocr_profile(const std::string& key);

}  // namespace uocr

