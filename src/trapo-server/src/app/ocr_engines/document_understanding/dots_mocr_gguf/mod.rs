use crate::app::ocr_engines::{
    RunnerCapability, RunnerResolveContext,
    common::{
        gguf_vlm::{GgufVlmSpec, resolve_runner},
        process_runner::EngineRunner,
    },
};

pub(in crate::app::ocr_engines) const ENGINE_ID: &str = "dots-mocr-gguf";
pub(in crate::app::ocr_engines) const RUNNER_NAMES: &[&str] = &["llama-mtmd-cli"];
pub(in crate::app::ocr_engines) const EXPECTED_BINARY: &str = "llama-mtmd-cli";

const SPEC: GgufVlmSpec = GgufVlmSpec {
    engine_id: ENGINE_ID,
    default_model_id: "dots-mocr-gguf",
    prompt: "OCR",
    max_tokens: 4096,
    chat_template: None,
};

pub(in crate::app::ocr_engines) const fn capability() -> RunnerCapability {
    RunnerCapability {
        kind: "gguf-vlm-native",
        status: "wired",
        detail: Some("uses llama-mtmd-cli with dots.ocr GGUF and mmproj assets"),
    }
}

pub(in crate::app::ocr_engines) fn resolve(
    context: &RunnerResolveContext<'_>,
) -> std::result::Result<EngineRunner, String> {
    resolve_runner(context, &SPEC)
}
