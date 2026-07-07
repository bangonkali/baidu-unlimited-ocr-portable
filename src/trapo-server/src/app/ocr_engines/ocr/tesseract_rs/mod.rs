use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::missing_native_runner_binary,
        process_runner::{EngineRunner, RunnerKind},
        runtime_search::find_runner_binary,
    },
};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "tesseract-rs";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] =
    &["trapo-tesseract-rs-runner", "tesseract"];
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str =
    "trapo-tesseract-rs-runner plus tesseract/bin/tesseract and tessdata";
const ENGINE_ASSET_DIR: &str = "tesseract";
const TESSDATA_FILE: &str = "eng.traineddata";

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "tesseract-native",
        status: "wired",
        detail: Some("uses trapo-tesseract-rs-runner when present, otherwise tesseract CLI"),
    }
}

pub(in crate::app::ocr_engines) fn resolve(
    context: &RunnerResolveContext<'_>,
) -> std::result::Result<EngineRunner, String> {
    if let Some(found) = find_runner_binary(
        context.app_root,
        context.runtime_id,
        &["trapo-tesseract-rs-runner"],
    ) {
        if std::env::var_os("TRAPO_TESSERACT_COMMAND").is_none()
            && !engine_assets_are_installed(found.runtime_bin_dir.as_deref())
        {
            return Err(format!(
                "{ENGINE_ID} engine assets are not installed; expected {ENGINE_ASSET_DIR}/bin/tesseract and {ENGINE_ASSET_DIR}/tessdata/{TESSDATA_FILE} next to the selected runtime bin directory"
            ));
        }
        return Ok(EngineRunner {
            engine_id: ENGINE_ID.to_string(),
            command: found.path,
            runtime_bin_dir: found.runtime_bin_dir,
            kind: RunnerKind::GenericJsonText {
                args: vec!["--format".to_string(), "text".to_string()],
            },
        });
    }
    let found = find_runner_binary(context.app_root, context.runtime_id, &["tesseract"])
        .ok_or_else(|| missing_native_runner_binary(ENGINE_ID, EXPECTED_BINARY))?;
    Ok(EngineRunner {
        engine_id: ENGINE_ID.to_string(),
        command: found.path,
        runtime_bin_dir: found.runtime_bin_dir,
        kind: RunnerKind::TesseractCli {
            language: "eng".to_string(),
            page_segmentation_mode: "6".to_string(),
        },
    })
}

fn engine_assets_are_installed(runtime_bin_dir: Option<&std::path::Path>) -> bool {
    let Some(root) = runtime_bin_dir.and_then(std::path::Path::parent) else {
        return false;
    };
    let command = if cfg!(windows) {
        root.join(ENGINE_ASSET_DIR)
            .join("bin")
            .join("tesseract.exe")
    } else {
        root.join(ENGINE_ASSET_DIR).join("bin").join("tesseract")
    };
    command.is_file()
        && root
            .join(ENGINE_ASSET_DIR)
            .join("tessdata")
            .join(TESSDATA_FILE)
            .is_file()
}
