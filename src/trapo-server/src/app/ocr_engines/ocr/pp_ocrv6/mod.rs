use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::missing_native_runner_binary,
        process_runner::{EngineRunner, RunnerKind},
        runtime_search::find_runner_binary,
    },
};
use std::{fs, path::Path};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "pp-ocrv6";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] =
    &["trapo-pp-ocrv6-runner", "pp-ocrv6-runner"];
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str = "trapo-pp-ocrv6-runner";
const ENGINE_ASSET_DIR: &str = "ppocrv6";
const MODEL_MANIFEST: &str = "models/manifest.json";

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "ppocrv6-native-ffi",
        status: "wired",
        detail: Some("uses trapo-ocr-ffi with ONNX Runtime, OpenCV, and Clipper"),
    }
}

pub(in crate::app::ocr_engines) fn resolve(
    context: &RunnerResolveContext<'_>,
) -> std::result::Result<EngineRunner, String> {
    let found = find_runner_binary(context.app_root, context.runtime_id, RUNNER_NAMES)
        .ok_or_else(|| missing_native_runner_binary(ENGINE_ID, EXPECTED_BINARY))?;
    validate_engine_assets_installed(found.runtime_bin_dir.as_deref())?;
    Ok(EngineRunner {
        engine_id: ENGINE_ID.to_string(),
        command: found.path,
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::GenericJsonText {
            args: vec!["--format".to_string(), "text".to_string()],
        },
    })
}

fn validate_engine_assets_installed(runtime_bin_dir: Option<&Path>) -> Result<(), String> {
    let Some(root) = runtime_bin_dir.and_then(std::path::Path::parent) else {
        return Err(format!(
            "{ENGINE_ID} requires a packaged runtime bin directory"
        ));
    };
    let engine_root = root.join(ENGINE_ASSET_DIR);
    if !engine_root.join(MODEL_MANIFEST).is_file() || !shared_ffi_is_installed(root, &engine_root) {
        return Err(format!(
            "{ENGINE_ID} engine assets are not installed; expected {ENGINE_ASSET_DIR}/{MODEL_MANIFEST} and shared bin/trapo-ocr-ffi next to the selected runtime bin directory"
        ));
    }
    if contains_python_runtime_assets(&engine_root) {
        return Err(format!(
            "{ENGINE_ID} runtime contains stale Python-era PP-OCRv6 assets under {}; rebuild the native runtime bundle",
            engine_root.display()
        ));
    }
    Ok(())
}

fn shared_ffi_is_installed(runtime_root: &std::path::Path, engine_root: &std::path::Path) -> bool {
    ffi_library_names().iter().any(|name| {
        runtime_root.join("bin").join(name).is_file()
            || engine_root.join("bin").join(name).is_file()
    })
}

fn contains_python_runtime_assets(home: &Path) -> bool {
    contains_forbidden_asset(home, home)
}

fn contains_forbidden_asset(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if is_forbidden_ppocrv6_asset(relative) {
        return true;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return true;
    };
    if !metadata.is_dir() {
        return false;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return true;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            return true;
        };
        if contains_forbidden_asset(root, &entry.path()) {
            return true;
        }
    }
    false
}

fn is_forbidden_ppocrv6_asset(relative: &Path) -> bool {
    let filename = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if filename.starts_with("trapo_ppocrv6_engine") || has_forbidden_extension(relative) {
        return true;
    }
    relative.components().any(|component| {
        let name = component
            .as_os_str()
            .to_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        matches!(
            name.as_str(),
            ".venv" | ".paddlex" | "__pycache__" | "build" | "ppocrv6"
        ) || name.starts_with("trapo_ppocrv6_engine")
    })
}

fn has_forbidden_extension(relative: &Path) -> bool {
    relative
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "py" | "pyc" | "pyo" | "pyd" | "spec"
            )
        })
}

const fn ffi_library_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["trapo-ocr-ffi.dll", "agus_ocr_core.dll"]
    } else if cfg!(target_os = "macos") {
        &["libtrapo-ocr-ffi.dylib", "libagus_ocr_core.dylib"]
    } else {
        &["libtrapo-ocr-ffi.so", "libagus_ocr_core.so"]
    }
}
