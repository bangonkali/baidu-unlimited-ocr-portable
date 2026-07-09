use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::missing_native_runner_binary,
        model_bundles,
        native_ocr_ffi::{NativeOcrFfiConfig, NativeOcrPipeline, NativeOcrRuntimeConfig},
        onnx_runtime,
        process_runner::{EngineRunner, RunnerKind},
        runtime_search::find_runner_binary,
    },
};
use std::{fs, path::Path};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "pp-ocrv6";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] = ffi_library_names();
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str = "trapo-ocr-ffi";
const ENGINE_ASSET_DIR: &str = "ppocrv6";

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "ppocrv6-onnx-ffi",
        status: "wired",
        detail: Some("uses in-process trapo-ocr-ffi with ONNX Runtime, OpenCV, and Clipper"),
    }
}

pub(in crate::app::ocr_engines) fn resolve(
    context: &RunnerResolveContext<'_>,
) -> std::result::Result<EngineRunner, String> {
    let found = find_runner_binary(context.app_root, context.runtime_id, RUNNER_NAMES)
        .ok_or_else(|| missing_native_runner_binary(ENGINE_ID, EXPECTED_BINARY))?;
    let runtime_root = found
        .runtime_bin_dir
        .as_deref()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| format!("{ENGINE_ID} requires a packaged runtime bin directory"))?;
    let _onnxruntime = onnx_runtime::validate_for_native_library(&found.path)?;
    let bundle_root = runtime_root.join(ENGINE_ASSET_DIR);
    validate_engine_assets_installed(&bundle_root)?;
    let model_root = bundle_root.join("models");
    Ok(EngineRunner {
        engine_id: ENGINE_ID.to_string(),
        command: found.path.clone(),
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::NativeOcrFfi {
            config: NativeOcrFfiConfig {
                pipeline: NativeOcrPipeline::PpOcrV6,
                library_path: found.path,
                model_root,
                external_model_root: None,
                vl_model_path: None,
                vl_mmproj_path: None,
                runtime: NativeOcrRuntimeConfig::from_runtime_id(context.runtime_id),
                max_new_tokens: 0,
                generate_markdown: false,
            },
        },
    })
}

fn validate_engine_assets_installed(engine_root: &Path) -> Result<(), String> {
    model_bundles::ppocrv6(engine_root).ensure_available()?;
    if contains_python_runtime_assets(engine_root) {
        return Err(format!(
            "{ENGINE_ID} runtime contains stale Python-era PP-OCRv6 assets under {}; rebuild the native runtime bundle",
            engine_root.display()
        ));
    }
    Ok(())
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
        &["trapo-ocr-ffi.dll"]
    } else if cfg!(target_os = "macos") {
        &["libtrapo-ocr-ffi.dylib"]
    } else {
        &["libtrapo-ocr-ffi.so"]
    }
}
