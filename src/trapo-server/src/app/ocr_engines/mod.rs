use std::path::Path;

use super::{AppState, OcrRunWorker};

mod common;
mod document_understanding;
mod ocr;

pub(super) use common::process_runner::EngineRunner;

#[derive(Clone, Copy)]
pub(super) struct RunnerCapability {
    pub(super) kind: &'static str,
    pub(super) status: &'static str,
    pub(super) detail: Option<&'static str>,
}

pub(super) struct RunnerResolveContext<'a> {
    pub(super) app_root: &'a Path,
    pub(super) model_dir: &'a Path,
    pub(super) runtime_id: &'a str,
    pub(super) model_id: &'a str,
}

impl AppState {
    pub(super) async fn create_adapter_ocr_worker(
        &self,
        engine_id: &str,
        runtime_id: &str,
        model_id: &str,
    ) -> OcrRunWorker {
        match self.adapter_runner(engine_id, runtime_id, model_id).await {
            Ok(runner) => OcrRunWorker::spawn_adapter(
                runner,
                self.inner.hub.clone(),
                self.inner.annotation_identities.clone(),
            ),
            Err(error) => OcrRunWorker::fallback(error),
        }
    }

    async fn adapter_runner(
        &self,
        engine_id: &str,
        runtime_id: &str,
        model_id: &str,
    ) -> std::result::Result<EngineRunner, String> {
        let runtime_id = self.adapter_runtime_id(runtime_id).await;
        let context = RunnerResolveContext {
            app_root: &self.inner.config.app_root,
            model_dir: &self.inner.config.model_dir,
            runtime_id: &runtime_id,
            model_id,
        };
        match engine_id {
            ocr::tesseract_rs::ENGINE_ID => ocr::tesseract_rs::resolve(&context),
            ocr::pp_ocrv6::ENGINE_ID => ocr::pp_ocrv6::resolve(&context),
            ocr::paddleocr_vl_1_6_gguf::ENGINE_ID => ocr::paddleocr_vl_1_6_gguf::resolve(&context),
            document_understanding::dots_mocr_gguf::ENGINE_ID => {
                document_understanding::dots_mocr_gguf::resolve(&context)
            }
            document_understanding::infinity_parser2_flash_gguf::ENGINE_ID => {
                document_understanding::infinity_parser2_flash_gguf::resolve(&context)
            }
            _ => Err(format!(
                "no native OCR runner is registered for engine {engine_id}"
            )),
        }
    }

    async fn adapter_runtime_id(&self, runtime_id: &str) -> String {
        if !runtime_id.is_empty() {
            return runtime_id.to_string();
        }
        self.inner.state.lock().await.selected_runtime_id.clone()
    }
}

pub(super) fn runner_capability(engine_id: &str) -> RunnerCapability {
    match engine_id {
        super::ENGINE_ID => RunnerCapability {
            kind: "llama.cpp-ffi",
            status: "ready",
            detail: None,
        },
        ocr::tesseract_rs::ENGINE_ID => ocr::tesseract_rs::capability(),
        ocr::pp_ocrv6::ENGINE_ID => ocr::pp_ocrv6::capability(),
        ocr::paddleocr_vl_1_6_gguf::ENGINE_ID => ocr::paddleocr_vl_1_6_gguf::capability(),
        document_understanding::dots_mocr_gguf::ENGINE_ID => {
            document_understanding::dots_mocr_gguf::capability()
        }
        document_understanding::infinity_parser2_flash_gguf::ENGINE_ID => {
            document_understanding::infinity_parser2_flash_gguf::capability()
        }
        _ => RunnerCapability {
            kind: "native",
            status: "unknown",
            detail: Some("engine is not registered"),
        },
    }
}

pub(super) fn runner_binary_is_installed(
    app_root: &Path,
    runtime_id: &str,
    engine_id: &str,
) -> bool {
    let Some(names) = runner_binary_names(engine_id) else {
        return false;
    };
    common::runtime_search::runner_binary_is_installed(app_root, runtime_id, names)
}

pub(super) fn expected_runner_binary(engine_id: &str) -> Option<&'static str> {
    match engine_id {
        ocr::tesseract_rs::ENGINE_ID => Some(ocr::tesseract_rs::EXPECTED_BINARY),
        ocr::pp_ocrv6::ENGINE_ID => Some(ocr::pp_ocrv6::EXPECTED_BINARY),
        ocr::paddleocr_vl_1_6_gguf::ENGINE_ID => Some(ocr::paddleocr_vl_1_6_gguf::EXPECTED_BINARY),
        document_understanding::dots_mocr_gguf::ENGINE_ID => {
            Some(document_understanding::dots_mocr_gguf::EXPECTED_BINARY)
        }
        document_understanding::infinity_parser2_flash_gguf::ENGINE_ID => {
            Some(document_understanding::infinity_parser2_flash_gguf::EXPECTED_BINARY)
        }
        super::ENGINE_ID => None,
        _ => None,
    }
}

fn runner_binary_names(engine_id: &str) -> Option<&'static [&'static str]> {
    match engine_id {
        ocr::tesseract_rs::ENGINE_ID => Some(ocr::tesseract_rs::RUNNER_NAMES),
        ocr::pp_ocrv6::ENGINE_ID => Some(ocr::pp_ocrv6::RUNNER_NAMES),
        ocr::paddleocr_vl_1_6_gguf::ENGINE_ID => Some(ocr::paddleocr_vl_1_6_gguf::RUNNER_NAMES),
        document_understanding::dots_mocr_gguf::ENGINE_ID => {
            Some(document_understanding::dots_mocr_gguf::RUNNER_NAMES)
        }
        document_understanding::infinity_parser2_flash_gguf::ENGINE_ID => {
            Some(document_understanding::infinity_parser2_flash_gguf::RUNNER_NAMES)
        }
        _ => None,
    }
}
