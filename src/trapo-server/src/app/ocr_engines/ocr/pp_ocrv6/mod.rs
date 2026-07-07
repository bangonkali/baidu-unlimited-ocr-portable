use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::missing_native_runner_binary,
        process_runner::{EngineRunner, RunnerKind},
        runtime_search::find_runner_binary,
    },
};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "pp-ocrv6";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] =
    &["trapo-pp-ocrv6-runner", "pp-ocrv6-runner"];
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str = "trapo-pp-ocrv6-runner";
const ENGINE_ASSET_DIR: &str = "ppocrv6";
const ENGINE_SCRIPT: &str = "trapo_ppocrv6_engine.py";
const MODEL_MANIFEST: &str = "models/manifest.json";

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "paddleocr-native",
        status: "wired",
        detail: Some("uses a staged PP-OCRv6 native runner process"),
    }
}

pub(in crate::app::ocr_engines) fn resolve(
    context: &RunnerResolveContext<'_>,
) -> std::result::Result<EngineRunner, String> {
    let found = find_runner_binary(context.app_root, context.runtime_id, RUNNER_NAMES)
        .ok_or_else(|| missing_native_runner_binary(ENGINE_ID, EXPECTED_BINARY))?;
    if std::env::var_os("TRAPO_PP_OCRV6_COMMAND").is_none()
        && !engine_assets_are_installed(found.runtime_bin_dir.as_deref())
    {
        return Err(format!(
            "{ENGINE_ID} engine assets are not installed; expected {ENGINE_ASSET_DIR}/{ENGINE_SCRIPT} next to the selected runtime bin directory"
        ));
    }
    Ok(EngineRunner {
        engine_id: ENGINE_ID.to_string(),
        command: found.path,
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::GenericJsonText {
            args: vec!["--format".to_string(), "text".to_string()],
        },
    })
}

fn engine_assets_are_installed(runtime_bin_dir: Option<&std::path::Path>) -> bool {
    let Some(root) = runtime_bin_dir.and_then(std::path::Path::parent) else {
        return false;
    };
    let engine_root = root.join(ENGINE_ASSET_DIR);
    engine_root.join(ENGINE_SCRIPT).is_file()
        && engine_root.join(MODEL_MANIFEST).is_file()
        && (engine_root.join("bin").join(engine_binary_name()).is_file()
            || engine_root.join(embedded_python_path()).is_file())
}

const fn engine_binary_name() -> &'static str {
    if cfg!(windows) {
        "trapo_ppocrv6_engine.exe"
    } else {
        "trapo_ppocrv6_engine"
    }
}

const fn embedded_python_path() -> &'static str {
    if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    }
}
