pub fn ocr_profiles() -> Vec<OcrProfileRecord> {
    vec![
        OcrProfileRecord {
            key: RETRY_PROFILE_ID.to_string(),
            label: "Practical zero-empty Q4".to_string(),
            engine_name: "llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full".to_string(),
            description:
                "Current R-SWA Q4 demo default: 54/104 pass, zero empty rows, avg similarity 0.678."
                    .to_string(),
            default_max_tokens: 8192,
            ngram_size: 35,
            ngram_window: 128,
            pdf_ngram_window: 1024,
            force_prompt_eos: true,
            no_image_end: false,
        },
        OcrProfileRecord {
            key: DEFAULT_PROFILE_ID.to_string(),
            label: "Experimental exact-prefill Q4".to_string(),
            engine_name: "llamacpp-q4_k_m-uocr-rswa-noimgend-noeos-full".to_string(),
            description: "Higher avg similarity 0.719, but had 5 empty rows in full validation."
                .to_string(),
            default_max_tokens: 8192,
            ngram_size: 35,
            ngram_window: 128,
            pdf_ngram_window: 1024,
            force_prompt_eos: false,
            no_image_end: true,
        },
    ]
}

pub fn find_profile(profile_id: &str) -> Option<OcrProfileRecord> {
    ocr_profiles()
        .into_iter()
        .find(|profile| profile.key == profile_id)
}
