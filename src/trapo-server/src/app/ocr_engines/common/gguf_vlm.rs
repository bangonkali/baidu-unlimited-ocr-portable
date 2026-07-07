use std::path::Path;

use crate::catalog::find_model;

use super::{
    process_runner::{EngineRunner, RunnerKind},
    runtime_search::find_runner_binary,
};
use crate::app::ocr_engines::RunnerResolveContext;

pub(in crate::app::ocr_engines) struct GgufVlmSpec {
    pub(in crate::app::ocr_engines) engine_id: &'static str,
    pub(in crate::app::ocr_engines) default_model_id: &'static str,
    pub(in crate::app::ocr_engines) prompt: &'static str,
    pub(in crate::app::ocr_engines) max_tokens: u32,
    pub(in crate::app::ocr_engines) chat_template: Option<&'static str>,
}

pub(in crate::app::ocr_engines) fn resolve_runner(
    context: &RunnerResolveContext<'_>,
    spec: &GgufVlmSpec,
) -> std::result::Result<EngineRunner, String> {
    let found = find_runner_binary(context.app_root, context.runtime_id, &["llama-mtmd-cli"])
        .ok_or_else(|| missing_native_runner_binary(spec.engine_id, "llama-mtmd-cli"))?;
    let model_id = if context.model_id.is_empty() {
        spec.default_model_id
    } else {
        context.model_id
    };
    let model = find_model(model_id)
        .ok_or_else(|| format!("model was not found for {}: {model_id}", spec.engine_id))?;
    let model_path = context.model_dir.join(model.model_file);
    let mmproj_path = context.model_dir.join(
        model
            .mmproj_file
            .unwrap_or(crate::catalog::SHARED_MMPROJ_FILE),
    );
    validate_model_path(spec.engine_id, "model", &model_path)?;
    validate_model_path(spec.engine_id, "mmproj", &mmproj_path)?;
    Ok(EngineRunner {
        engine_id: spec.engine_id.to_string(),
        command: found.path,
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::LlamaMtmd {
            model: model_path,
            mmproj: mmproj_path,
            prompt: spec.prompt.to_string(),
            max_tokens: spec.max_tokens,
            chat_template: spec.chat_template.map(ToString::to_string),
        },
    })
}

fn validate_model_path(
    engine_id: &str,
    kind: &str,
    path: &Path,
) -> std::result::Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    Err(format!(
        "{kind} file is missing for {engine_id}: {}",
        path.display()
    ))
}

pub(in crate::app::ocr_engines) fn missing_native_runner_binary(
    engine_id: &str,
    expected: &str,
) -> String {
    format!(
        "{engine_id} native runner binary is not installed; expected {expected} in a runtime bin directory"
    )
}
