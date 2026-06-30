#include "uocr/core/profiles.hpp"

#include <stdexcept>

namespace uocr {

const std::vector<PromptProfileRecord>& prompt_profiles() {
  static const std::vector<PromptProfileRecord> profiles = {
      {"document_parsing", "Document parsing", "document parsing.",
       "Native Unlimited-OCR / DeepSeek-OCR parse prompt."},
      {"grounding", "Grounding markdown", "<|grounding|>Convert the document to markdown.",
       "Layout-aware markdown with detection markers."},
      {"plain_text", "Plain OCR", "Free OCR.", "Plain OCR text without explicit grounding markers."},
      {"ocr_boxes", "OCR boxes", "<|grounding|>OCR this image.",
       "Model-card OCR prompt with bounding boxes."},
  };
  return profiles;
}

const std::vector<OcrProfileRecord>& ocr_profiles() {
  static const std::vector<OcrProfileRecord> profiles = {
      {
          .key = "best-zero-empty-q4",
          .label = "Practical zero-empty Q4",
          .engine_name = "llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full",
          .description =
              "Current R-SWA Q4 demo default: 54/104 pass, zero empty rows, avg similarity 0.678.",
          .force_prompt_eos = true,
          .no_image_end = false,
      },
      {
          .key = "experimental-exact-prefill-q4",
          .label = "Experimental exact-prefill Q4",
          .engine_name = "llamacpp-q4_k_m-uocr-rswa-noimgend-noeos-full",
          .description = "Higher avg similarity 0.719, but had 5 empty rows in full validation.",
          .force_prompt_eos = false,
          .no_image_end = true,
      },
  };
  return profiles;
}

const OcrProfileRecord& default_ocr_profile() {
  const OcrProfileRecord* profile = find_ocr_profile("experimental-exact-prefill-q4");
  if (profile == nullptr) {
    throw std::logic_error("default OCR profile is not registered");
  }
  return *profile;
}

const OcrProfileRecord* find_ocr_profile(const std::string& key) {
  for (const auto& profile : ocr_profiles()) {
    if (profile.key == key) {
      return &profile;
    }
  }
  return nullptr;
}

}  // namespace uocr
