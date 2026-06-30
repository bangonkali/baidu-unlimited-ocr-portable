#include "uocr/core/model_catalog.hpp"

namespace uocr {

std::string_view default_model_id() {
  return "unlimited-ocr-q4-k-m";
}

std::string_view provider_repo_id() {
  return "sahilchachra/Unlimited-OCR-GGUF";
}

std::string_view provider_revision() {
  return "main";
}

std::string_view provider_label() {
  return "Sahil Chachra Unlimited-OCR GGUF";
}

std::string_view shared_mmproj_file() {
  return "mmproj-Unlimited-OCR-F16.gguf";
}

std::uint64_t shared_mmproj_size_bytes() {
  return 811876448ULL;
}

const std::vector<ModelCatalogEntry>& unlimited_ocr_model_catalog() {
  static const std::vector<ModelCatalogEntry> catalog = {
      {"unlimited-ocr-bf16", "Unlimited-OCR BF16", "Unlimited-OCR-BF16.gguf", "BF16",
       "Reference quality", "Very high VRAM", "Largest model; use for diagnostics on high-memory GPUs.", 16,
       5876578080ULL, false},
      {"unlimited-ocr-q8-0", "Unlimited-OCR Q8_0", "Unlimited-OCR-Q8_0.gguf", "Q8_0",
       "Near lossless", "High VRAM", "High quality with less memory than BF16.", 8, 3126139904ULL, false},
      {"unlimited-ocr-q6-k", "Unlimited-OCR Q6_K", "Unlimited-OCR-Q6_K.gguf", "Q6_K",
       "Very high quality", "Medium-high VRAM", "Good quality target when Q8 is too large.", 6, 2613275904ULL,
       false},
      {"unlimited-ocr-q5-k-m", "Unlimited-OCR Q5_K_M", "Unlimited-OCR-Q5_K_M.gguf", "Q5_K_M",
       "High quality", "Medium VRAM", "Balanced higher-quality option.", 5, 2219208704ULL, false},
      {"unlimited-ocr-q5-k-s", "Unlimited-OCR Q5_K_S", "Unlimited-OCR-Q5_K_S.gguf", "Q5_K_S",
       "High quality, smaller", "Medium VRAM", "Smaller Q5 variant.", 5, 2098952704ULL, false},
      {"unlimited-ocr-q4-k-m", "Unlimited-OCR Q4_K_M", "Unlimited-OCR-Q4_K_M.gguf", "Q4_K_M",
       "Recommended balance", "Most CUDA GPUs", "Default practical size and quality choice.", 4, 1950326784ULL,
       true},
      {"unlimited-ocr-q4-k-s", "Unlimited-OCR Q4_K_S", "Unlimited-OCR-Q4_K_S.gguf", "Q4_K_S",
       "Smaller Q4", "Most CUDA GPUs", "Smaller Q4 option with modest quality cost.", 4, 1805289984ULL, false},
      {"unlimited-ocr-iq4-nl", "Unlimited-OCR IQ4_NL", "Unlimited-OCR-IQ4_NL.gguf", "IQ4_NL",
       "Edge tuned", "Most CUDA GPUs", "I-quant variant tuned for edge and ARM-style targets.", 4, 1701901824ULL,
       false},
      {"unlimited-ocr-iq4-xs", "Unlimited-OCR IQ4_XS", "Unlimited-OCR-IQ4_XS.gguf", "IQ4_XS",
       "Compact Q4", "Most CUDA GPUs", "Smaller I-quant Q4 option.", 4, 1640897024ULL, false},
      {"unlimited-ocr-q3-k-m", "Unlimited-OCR Q3_K_M", "Unlimited-OCR-Q3_K_M.gguf", "Q3_K_M",
       "Compact", "Tight memory", "Use when Q4 variants do not fit.", 3, 1553635584ULL, false},
      {"unlimited-ocr-iq3-m", "Unlimited-OCR IQ3_M", "Unlimited-OCR-IQ3_M.gguf", "IQ3_M",
       "Compact 3-bit", "Tight memory", "I-quant 3-bit option.", 3, 1448949504ULL, false},
      {"unlimited-ocr-iq3-xxs", "Unlimited-OCR IQ3_XXS", "Unlimited-OCR-IQ3_XXS.gguf", "IQ3_XXS",
       "Very small", "Very tight memory", "Very small model with visible quality loss.", 3, 1335367424ULL, false},
      {"unlimited-ocr-iq2-m", "Unlimited-OCR IQ2_M", "Unlimited-OCR-IQ2_M.gguf", "IQ2_M",
       "Smallest experimental", "Very tight memory", "Smallest option; quality tradeoffs are expected.", 2,
       1232148224ULL, false},
  };
  return catalog;
}

const ModelCatalogEntry* find_model_catalog_entry(std::string_view model_id) {
  for (const auto& entry : unlimited_ocr_model_catalog()) {
    if (entry.model_id == model_id) {
      return &entry;
    }
  }
  return nullptr;
}

}  // namespace uocr
