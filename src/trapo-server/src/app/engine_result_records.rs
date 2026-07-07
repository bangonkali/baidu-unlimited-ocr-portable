fn engine_config_record(config: &RunEngineConfigState) -> IngestEngineConfigRecord {
    IngestEngineConfigRecord {
        run_engine_id: config.run_engine_id.clone(),
        run_id: config.run_id.clone(),
        ordinal: config.ordinal,
        engine_kind: config.engine_kind.clone(),
        engine_id: config.engine_id.clone(),
        label: engine_label(&config.engine_id),
        model_id: config.model_id.clone(),
        profile_id: config.profile_id.clone(),
        runtime_id: config.runtime_id.clone(),
        parameters: config.parameters.clone(),
        status: config.status.clone(),
        error: config.error.clone(),
        usable_output_count: config.usable_output_count,
        previewer: previewer_for_engine_kind(&config.engine_kind),
    }
}

fn preview_result_from_engine_config(config: &RunEngineConfigState) -> IngestPreviewResultRecord {
    IngestPreviewResultRecord {
        run_engine_id: config.run_engine_id.clone(),
        run_id: config.run_id.clone(),
        ordinal: config.ordinal,
        engine_kind: config.engine_kind.clone(),
        engine_id: config.engine_id.clone(),
        label: engine_label(&config.engine_id),
        model_id: config.model_id.clone(),
        profile_id: config.profile_id.clone(),
        runtime_id: config.runtime_id.clone(),
        status: config.status.clone(),
        previewer: previewer_for_engine_kind(&config.engine_kind),
        output_count: config.usable_output_count,
        page_count: config.usable_output_count,
        error: config.error.clone(),
    }
}

fn preview_result_record(row: StoredPreviewResult) -> IngestPreviewResultRecord {
    IngestPreviewResultRecord {
        run_engine_id: row.run_engine_id,
        run_id: row.run_id,
        ordinal: row.ordinal,
        engine_kind: row.engine_kind.clone(),
        engine_id: row.engine_id.clone(),
        label: engine_label(&row.engine_id),
        model_id: row.model_id,
        profile_id: row.profile_id,
        runtime_id: row.runtime_id,
        status: row.status,
        previewer: previewer_for_engine_kind(&row.engine_kind),
        output_count: row.output_count,
        page_count: row.page_count,
        error: row.error,
    }
}

fn run_engine_config_from_stored(stored: StoredRunEngineConfig) -> RunEngineConfigState {
    RunEngineConfigState {
        run_engine_id: stored.run_engine_id,
        run_id: stored.run_id,
        ordinal: stored.ordinal,
        engine_kind: stored.engine_kind,
        engine_id: stored.engine_id,
        model_id: stored.model_id,
        profile_id: stored.profile_id,
        runtime_id: stored.runtime_id,
        parameters: stored.parameters,
        status: stored.status,
        error: stored.error,
        usable_output_count: stored.usable_output_count,
    }
}

fn stored_run_engine_config(config: &RunEngineConfigState) -> StoredRunEngineConfig {
    StoredRunEngineConfig {
        run_engine_id: config.run_engine_id.clone(),
        run_id: config.run_id.clone(),
        ordinal: config.ordinal,
        engine_kind: config.engine_kind.clone(),
        engine_id: config.engine_id.clone(),
        model_id: config.model_id.clone(),
        profile_id: config.profile_id.clone(),
        runtime_id: config.runtime_id.clone(),
        parameters: config.parameters.clone(),
        status: config.status.clone(),
        error: config.error.clone(),
        usable_output_count: config.usable_output_count,
    }
}

fn legacy_engine_config_for_run(run: &RunState) -> RunEngineConfigState {
    RunEngineConfigState {
        run_engine_id: new_persistence_id(),
        run_id: run.run_id.clone(),
        ordinal: 0,
        engine_kind: "ocr".to_string(),
        engine_id: run.engine_id.clone(),
        model_id: Some(run.model_id.clone()),
        profile_id: Some(run.profile_id.clone()),
        runtime_id: Some(run.runtime_id.clone()),
        parameters: json!({}),
        status: run.status.clone(),
        error: run.error.clone(),
        usable_output_count: run.processed_pages,
    }
}

fn previewer_for_engine_kind(engine_kind: &str) -> String {
    if engine_kind == "document_understanding" {
        "document_markdown".to_string()
    } else {
        "ocr_annotation".to_string()
    }
}

fn engine_label(engine_id: &str) -> String {
    match engine_id {
        "unlimited-ocr-ffi" => "Unlimited OCR".to_string(),
        "tesseract-rs" => "Tesseract".to_string(),
        "pp-ocrv6" => "PP-OCRv6".to_string(),
        "paddleocr-vl-1.6-gguf" => "PaddleOCR-VL 1.6".to_string(),
        "dots-mocr-gguf" => "dots.mocr".to_string(),
        "infinity-parser2-flash-gguf" => "Infinity Parser2 Flash".to_string(),
        _ => engine_id.to_string(),
    }
}
