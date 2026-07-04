const MODEL_CATALOG: [ModelCatalogEntry; 13] = [
    ModelCatalogEntry::new(
        "unlimited-ocr-bf16",
        "Unlimited-OCR BF16",
        "Unlimited-OCR-BF16.gguf",
        "BF16",
        "Reference quality",
        "Very high VRAM",
        "Largest model; use for diagnostics on high-memory GPUs.",
        16,
        5_876_578_080,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q8-0",
        "Unlimited-OCR Q8_0",
        "Unlimited-OCR-Q8_0.gguf",
        "Q8_0",
        "Near lossless",
        "High VRAM",
        "High quality with less memory than BF16.",
        8,
        3_126_139_904,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q6-k",
        "Unlimited-OCR Q6_K",
        "Unlimited-OCR-Q6_K.gguf",
        "Q6_K",
        "Very high quality",
        "Medium-high VRAM",
        "Good quality target when Q8 is too large.",
        6,
        2_613_275_904,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q5-k-m",
        "Unlimited-OCR Q5_K_M",
        "Unlimited-OCR-Q5_K_M.gguf",
        "Q5_K_M",
        "High quality",
        "Medium VRAM",
        "Balanced higher-quality option.",
        5,
        2_219_208_704,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q5-k-s",
        "Unlimited-OCR Q5_K_S",
        "Unlimited-OCR-Q5_K_S.gguf",
        "Q5_K_S",
        "High quality, smaller",
        "Medium VRAM",
        "Smaller Q5 variant.",
        5,
        2_098_952_704,
        false,
    ),
    ModelCatalogEntry::new(
        DEFAULT_MODEL_ID,
        "Unlimited-OCR Q4_K_M",
        "Unlimited-OCR-Q4_K_M.gguf",
        "Q4_K_M",
        "Recommended balance",
        "Most CUDA GPUs",
        "Default practical size and quality choice.",
        4,
        1_950_326_784,
        true,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q4-k-s",
        "Unlimited-OCR Q4_K_S",
        "Unlimited-OCR-Q4_K_S.gguf",
        "Q4_K_S",
        "Smaller Q4",
        "Most CUDA GPUs",
        "Smaller Q4 option with modest quality cost.",
        4,
        1_805_289_984,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-iq4-nl",
        "Unlimited-OCR IQ4_NL",
        "Unlimited-OCR-IQ4_NL.gguf",
        "IQ4_NL",
        "Edge tuned",
        "Most CUDA GPUs",
        "I-quant variant tuned for edge and ARM-style targets.",
        4,
        1_701_901_824,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-iq4-xs",
        "Unlimited-OCR IQ4_XS",
        "Unlimited-OCR-IQ4_XS.gguf",
        "IQ4_XS",
        "Compact Q4",
        "Most CUDA GPUs",
        "Smaller I-quant Q4 option.",
        4,
        1_640_897_024,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-q3-k-m",
        "Unlimited-OCR Q3_K_M",
        "Unlimited-OCR-Q3_K_M.gguf",
        "Q3_K_M",
        "Compact",
        "Tight memory",
        "Use when Q4 variants do not fit.",
        3,
        1_553_635_584,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-iq3-m",
        "Unlimited-OCR IQ3_M",
        "Unlimited-OCR-IQ3_M.gguf",
        "IQ3_M",
        "Compact 3-bit",
        "Tight memory",
        "I-quant 3-bit option.",
        3,
        1_448_949_504,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-iq3-xxs",
        "Unlimited-OCR IQ3_XXS",
        "Unlimited-OCR-IQ3_XXS.gguf",
        "IQ3_XXS",
        "Very small",
        "Very tight memory",
        "Very small model with visible quality loss.",
        3,
        1_335_367_424,
        false,
    ),
    ModelCatalogEntry::new(
        "unlimited-ocr-iq2-m",
        "Unlimited-OCR IQ2_M",
        "Unlimited-OCR-IQ2_M.gguf",
        "IQ2_M",
        "Smallest experimental",
        "Very tight memory",
        "Smallest option; quality tradeoffs are expected.",
        2,
        1_232_148_224,
        false,
    ),
];

#[derive(Debug, Clone)]
struct RuntimeSpec {
    platform: &'static str,
    label: &'static str,
    accelerator: &'static str,
    backend: &'static str,
    library_name: &'static str,
    priority: i32,
    n_gpu_layers: i32,
}

#[derive(Debug, Default)]
struct HardwareProbe {
    cuda: bool,
    rocm: bool,
    metal: bool,
}

#[must_use]
pub(crate) const fn model_catalog() -> &'static [ModelCatalogEntry] {
    &MODEL_CATALOG
}

impl ModelCatalogEntry {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        model_id: &'static str,
        display_name: &'static str,
        model_file: &'static str,
        quantization: &'static str,
        quality: &'static str,
        hardware_tier: &'static str,
        notes: &'static str,
        bits: u8,
        model_size_bytes: u64,
        recommended: bool,
    ) -> Self {
        Self {
            model_id,
            display_name,
            model_file,
            quantization,
            quality,
            hardware_tier,
            notes,
            bits,
            model_size_bytes,
            recommended,
        }
    }
}

#[must_use]
pub(crate) fn find_model(model_id: &str) -> Option<&'static ModelCatalogEntry> {
    model_catalog()
        .iter()
        .find(|entry| entry.model_id == model_id)
}
