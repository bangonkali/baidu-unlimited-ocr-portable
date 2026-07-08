use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::missing_native_runner_binary,
        model_bundles,
        native_ocr_ffi::{NativeOcrFfiConfig, NativeOcrPipeline, NativeOcrRuntimeConfig},
        process_runner::{EngineRunner, RunnerKind},
        runtime_search::find_runner_binary,
    },
};
use crate::catalog::{SHARED_MMPROJ_FILE, find_model};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "paddleocr-vl-1.6-gguf";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] = ffi_library_names();
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str = "trapo-ocr-ffi";
const DEFAULT_MODEL_ID: &str = "paddleocr-vl-1-6-gguf";
const ENGINE_ASSET_DIR: &str = "paddleocr_vl_1_6";

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "gguf-vlm-native-ffi",
        status: "wired",
        detail: Some("uses in-process trapo-ocr-ffi with PaddleOCR-VL GGUF and mmproj assets"),
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
    let bundle_root = runtime_root.join(ENGINE_ASSET_DIR);
    let model_id = if context.model_id.is_empty() {
        DEFAULT_MODEL_ID
    } else {
        context.model_id
    };
    let model = find_model(model_id)
        .ok_or_else(|| format!("model was not found for {ENGINE_ID}: {model_id}"))?;
    let model_path = context.model_dir.join(model.model_file);
    let mmproj_path = context
        .model_dir
        .join(model.mmproj_file.unwrap_or(SHARED_MMPROJ_FILE));
    model_bundles::paddleocr_vl_1_6(&bundle_root, &model_path, &mmproj_path).ensure_available()?;
    Ok(EngineRunner {
        engine_id: ENGINE_ID.to_string(),
        command: found.path.clone(),
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::NativeOcrFfi {
            config: NativeOcrFfiConfig {
                pipeline: NativeOcrPipeline::PaddleOcrVl16,
                library_path: found.path,
                model_root: bundle_root,
                external_model_root: None,
                vl_model_path: Some(model_path),
                vl_mmproj_path: Some(mmproj_path),
                runtime: NativeOcrRuntimeConfig::from_runtime_id(context.runtime_id),
                max_new_tokens: 4096,
                generate_markdown: true,
            },
        },
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
