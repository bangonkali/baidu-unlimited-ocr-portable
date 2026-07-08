use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

use super::native_ocr_ffi::{self, NativeOcrFfiConfig};

#[derive(Debug, Clone)]
pub(in crate::app) struct EngineRunner {
    pub(in crate::app::ocr_engines) engine_id: String,
    pub(in crate::app::ocr_engines) command: PathBuf,
    pub(in crate::app::ocr_engines) runtime_bin_dir: Option<PathBuf>,
    pub(in crate::app::ocr_engines) kind: RunnerKind,
}

#[derive(Debug, Clone)]
pub(in crate::app::ocr_engines) enum RunnerKind {
    GenericJsonText {
        args: Vec<String>,
    },
    TesseractCli {
        language: String,
        page_segmentation_mode: String,
    },
    LlamaMtmd {
        model: PathBuf,
        mmproj: PathBuf,
        prompt: String,
        max_tokens: u32,
        chat_template: Option<String>,
    },
    NativeOcrFfi {
        config: NativeOcrFfiConfig,
    },
}

impl EngineRunner {
    pub(in crate::app) fn engine_id(&self) -> &str {
        &self.engine_id
    }

    pub(in crate::app) fn recognize(&self, image_path: &Path) -> crate::ocr::OcrResult {
        if !image_path.is_file() {
            return ocr_failure("image path does not exist");
        }
        if let RunnerKind::NativeOcrFfi { config } = &self.kind {
            return native_ocr_ffi::recognize_image(config, image_path);
        }
        let mut command = Command::new(&self.command);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        self.apply_runtime_path(&mut command);
        self.push_arguments(&mut command, image_path);
        match command.output() {
            Ok(output) => adapter_output_result(&self.engine_id, &output),
            Err(error) => ocr_failure(format!(
                "failed to start {} native runner {}: {error}",
                self.engine_id,
                self.command.display()
            )),
        }
    }

    fn apply_runtime_path(&self, command: &mut Command) {
        let Some(runtime_bin_dir) = &self.runtime_bin_dir else {
            return;
        };
        let mut paths = vec![runtime_bin_dir.clone()];
        if let Some(current) = env::var_os("PATH") {
            paths.extend(env::split_paths(&current));
        }
        if let Ok(joined) = env::join_paths(paths) {
            command.env("PATH", joined);
        }
    }

    fn push_arguments(&self, command: &mut Command, image_path: &Path) {
        match &self.kind {
            RunnerKind::GenericJsonText { args } => {
                command.args(args);
                command.arg("--image").arg(image_path);
            }
            RunnerKind::TesseractCli {
                language,
                page_segmentation_mode,
            } => {
                command
                    .arg(image_path)
                    .arg("stdout")
                    .arg("-l")
                    .arg(language)
                    .arg("--psm")
                    .arg(page_segmentation_mode);
            }
            RunnerKind::LlamaMtmd {
                model,
                mmproj,
                prompt,
                max_tokens,
                chat_template,
            } => {
                command
                    .arg("-m")
                    .arg(model)
                    .arg("--mmproj")
                    .arg(mmproj)
                    .arg("--image")
                    .arg(image_path)
                    .arg("-p")
                    .arg(prompt)
                    .arg("--fit")
                    .arg("off")
                    .arg("--no-warmup")
                    .arg("--temp")
                    .arg("0")
                    .arg("-n")
                    .arg(max_tokens.to_string());
                if let Some(chat_template) = chat_template {
                    command.arg("--chat-template").arg(chat_template);
                }
            }
            RunnerKind::NativeOcrFfi { .. } => {}
        }
    }
}

fn adapter_output_result(engine_id: &str, output: &std::process::Output) -> crate::ocr::OcrResult {
    let stdout = clean_process_text(&String::from_utf8_lossy(&output.stdout));
    let stderr = clean_process_text(&String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return ocr_failure(format!(
            "{engine_id} native runner failed with exit code {}. stderr tail: {} stdout tail: {}",
            output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |code| code.to_string()),
            output_tail(&stderr),
            output_tail(&stdout)
        ));
    }
    if stdout.is_empty() {
        return ocr_failure(format!("{engine_id} native runner produced no text"));
    }
    if let Ok(payload) = serde_json::from_str::<Value>(&stdout)
        && payload.is_object()
    {
        let ok = payload
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let text = payload
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let error = payload
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string);
        return crate::ocr::OcrResult { ok, text, error };
    }
    crate::ocr::OcrResult {
        ok: true,
        text: stdout,
        error: None,
    }
}

fn ocr_failure(message: impl Into<String>) -> crate::ocr::OcrResult {
    crate::ocr::OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message.into()),
    }
}

fn clean_process_text(value: &str) -> String {
    strip_ansi(value).trim().to_string()
}

fn output_tail(value: &str) -> String {
    let tail = value
        .chars()
        .rev()
        .take(1_500)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    if tail.is_empty() {
        "<empty>".to_string()
    } else {
        tail
    }
}

fn strip_ansi(value: &str) -> String {
    static ANSI_PATTERN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]").unwrap_or_else(|_| std::process::abort())
    });
    ANSI_PATTERN.replace_all(value, "").into_owned()
}
