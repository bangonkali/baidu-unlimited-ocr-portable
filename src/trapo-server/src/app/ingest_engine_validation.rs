fn validate_engine_runner(preset: &EnginePresetDefinition) -> Result<()> {
    if runner_capability(preset.engine_id).status != "unknown" {
        return Ok(());
    }
    Err(AppError::BadRequest(format!(
        "unsupported ingest engine: {}",
        preset.engine_id
    )))
}

fn validate_engine_model(model_id: Option<&str>, engine_kind: &str) -> Result<()> {
    let Some(model_id) = model_id else {
        return Ok(());
    };
    let Some(model) = find_model(model_id) else {
        return Err(AppError::BadRequest(format!("unknown model id: {model_id}")));
    };
    let valid_kind = match engine_kind {
        "ocr" => model.model_kind == "ocr",
        "document_understanding" => model.model_kind == "document_understanding",
        _ => false,
    };
    if !valid_kind {
        return Err(AppError::BadRequest(format!(
            "model {model_id} is not valid for {engine_kind}"
        )));
    }
    Ok(())
}

fn validate_engine_profile(profile_id: Option<&str>) -> Result<()> {
    if let Some(profile_id) = profile_id
        && find_profile(profile_id).is_none()
    {
        return Err(AppError::BadRequest(format!(
            "unknown OCR profile: {profile_id}"
        )));
    }
    Ok(())
}

fn validate_engine_runtime(
    runtime_id: Option<&str>,
    variants: &[crate::catalog::RuntimeVariant],
) -> Result<()> {
    let Some(runtime_id) = runtime_id else {
        return Ok(());
    };
    if variants
        .iter()
        .any(|item| item.runtime_id == *runtime_id && item.selectable)
    {
        return Ok(());
    }
    Err(AppError::BadRequest(format!(
        "runtime is not supported on this device or is not installed: {runtime_id}"
    )))
}
