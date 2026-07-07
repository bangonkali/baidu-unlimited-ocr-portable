struct EnginePresetDefinition {
    preset_id: &'static str,
    engine_id: &'static str,
    engine_kind: &'static str,
    label: &'static str,
    description: &'static str,
    model_id: Option<&'static str>,
    profile_id: Option<&'static str>,
    previewer: &'static str,
    default_parameters: Value,
}

fn engine_preset_definitions() -> Vec<EnginePresetDefinition> {
    vec![
        EnginePresetDefinition {
            preset_id: "ocr-tesseract-rs",
            engine_id: "tesseract-rs",
            engine_kind: "ocr",
            label: "OCR: Tesseract",
            description: "Tesseract OCR adapter for baseline OCR comparison.",
            model_id: None,
            profile_id: None,
            previewer: "ocr_annotation",
            default_parameters: json!({ "language": "eng", "page_segmentation_mode": 6 }),
        },
        EnginePresetDefinition {
            preset_id: "ocr-unlimited-ocr-ffi",
            engine_id: ENGINE_ID,
            engine_kind: "ocr",
            label: "OCR: Unlimited OCR",
            description: "Existing llama.cpp Unlimited OCR FFI engine.",
            model_id: Some(DEFAULT_MODEL_ID),
            profile_id: Some(DEFAULT_PROFILE_ID),
            previewer: "ocr_annotation",
            default_parameters: json!({}),
        },
        EnginePresetDefinition {
            preset_id: "ocr-pp-ocrv6",
            engine_id: "pp-ocrv6",
            engine_kind: "ocr",
            label: "OCR: PP-OCRv6",
            description: "PaddleOCR PP-OCRv6 adapter target.",
            model_id: None,
            profile_id: None,
            previewer: "ocr_annotation",
            default_parameters: json!({ "language": "en" }),
        },
        EnginePresetDefinition {
            preset_id: "ocr-paddleocr-vl-1-6-gguf",
            engine_id: "paddleocr-vl-1.6-gguf",
            engine_kind: "ocr",
            label: "OCR: PaddleOCR-VL 1.6",
            description: "GGUF vision-language OCR/document hybrid target.",
            model_id: Some("paddleocr-vl-1-6-gguf"),
            profile_id: Some(DEFAULT_PROFILE_ID),
            previewer: "ocr_annotation",
            default_parameters: json!({ "prompt": "ocr" }),
        },
        EnginePresetDefinition {
            preset_id: "du-dots-mocr-gguf",
            engine_id: "dots-mocr-gguf",
            engine_kind: "document_understanding",
            label: "Document: dots.mocr",
            description: "dots.ocr GGUF document understanding target.",
            model_id: Some("dots-mocr-gguf"),
            profile_id: None,
            previewer: "document_markdown",
            default_parameters: json!({ "output": "markdown" }),
        },
        EnginePresetDefinition {
            preset_id: "du-infinity-parser2-flash-gguf",
            engine_id: "infinity-parser2-flash-gguf",
            engine_kind: "document_understanding",
            label: "Document: Infinity Parser2 Flash",
            description: "Infinity Parser2 Flash GGUF document understanding target.",
            model_id: Some("infinity-parser2-flash-gguf"),
            profile_id: None,
            previewer: "document_markdown",
            default_parameters: json!({ "output": "markdown" }),
        },
    ]
}

fn find_engine_preset(
    selection: &crate::workbench_types::IngestEngineSelection,
) -> Result<EnginePresetDefinition> {
    engine_preset_definitions()
        .into_iter()
        .find(|preset| {
            selection.preset_id.as_deref() == Some(preset.preset_id)
                || selection.engine_id == preset.engine_id
        })
        .ok_or_else(|| {
            AppError::BadRequest(format!("unknown ingest engine: {}", selection.engine_id))
        })
}
