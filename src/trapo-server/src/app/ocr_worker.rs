use std::{sync::mpsc, thread};

struct OcrRunWorker {
    state: OcrRunWorkerState,
}

enum OcrRunWorkerState {
    Ready {
        sender: mpsc::Sender<OcrWorkerMessage>,
        join: Option<thread::JoinHandle<()>>,
    },
    Fallback(String),
}

enum OcrWorkerMessage {
    Recognize(Box<OcrWorkerRequest>),
    Shutdown,
}

struct OcrWorkerRequest {
    image_path: PathBuf,
    context: OcrStreamContext,
    response: mpsc::Sender<crate::ocr::OcrResult>,
}

struct OcrRunContext<'a> {
    profile_id: &'a str,
    model_id: &'a str,
    runtime_id: &'a str,
    runtime_platform: &'a str,
    accelerator: &'a str,
    worker: &'a OcrRunWorker,
}

struct PageWork<'a> {
    run_id: &'a str,
    file_hash: &'a str,
    image_path: &'a Path,
    page_no: u32,
}

impl OcrRunWorker {
    fn fallback(reason: impl Into<String>) -> Self {
        Self {
            state: OcrRunWorkerState::Fallback(reason.into()),
        }
    }

    fn spawn(
        paths: crate::ocr::OcrRuntimePaths,
        profile: crate::types::OcrProfileRecord,
        hub: Arc<RealtimeHub>,
    ) -> Self {
        let (ready_sender, ready_receiver) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();
        let max_tokens = u32_to_i32_saturating(profile.default_max_tokens);
        let join = match thread::Builder::new()
            .name("trapo-ocr-worker".to_string())
            .spawn(move || {
                let mut engine = match crate::ocr::UnlimitedOcrFfiEngine::load(&paths, &profile) {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = ready_sender.send(Err(error.to_string()));
                        return;
                    }
                };
                let _ = ready_sender.send(Ok(()));
                run_worker_loop(&mut engine, receiver, max_tokens, &hub);
            }) {
            Ok(join) => join,
            Err(error) => {
                return Self::fallback(format!("could not start OCR worker thread: {error}"));
            }
        };
        match ready_receiver.recv() {
            Ok(Ok(())) => Self {
                state: OcrRunWorkerState::Ready {
                    sender,
                    join: Some(join),
                },
            },
            Ok(Err(error)) => {
                let _ = join.join();
                Self::fallback(error)
            }
            Err(error) => {
                let _ = join.join();
                Self::fallback(format!("OCR worker failed during startup: {error}"))
            }
        }
    }

    fn recognize(&self, image_path: &Path, context: OcrStreamContext) -> crate::ocr::OcrResult {
        let OcrRunWorkerState::Ready { sender, .. } = &self.state else {
            return ocr_failure(self.fallback_reason());
        };
        let (response, receiver) = mpsc::channel();
        let request = OcrWorkerRequest {
            image_path: image_path.to_path_buf(),
            context,
            response,
        };
        if sender
            .send(OcrWorkerMessage::Recognize(Box::new(request)))
            .is_err()
        {
            return ocr_failure("OCR worker is not running");
        }
        receiver
            .recv()
            .unwrap_or_else(|error| ocr_failure(format!("OCR worker did not return a result: {error}")))
    }

    fn fallback_reason(&self) -> String {
        match &self.state {
            OcrRunWorkerState::Ready { .. } => "OCR worker is running".to_string(),
            OcrRunWorkerState::Fallback(reason) => reason.clone(),
        }
    }

    fn fallback_error(&self) -> Option<&str> {
        match &self.state {
            OcrRunWorkerState::Ready { .. } => None,
            OcrRunWorkerState::Fallback(reason) => Some(reason),
        }
    }
}

impl Drop for OcrRunWorker {
    fn drop(&mut self) {
        if let OcrRunWorkerState::Ready { sender, join } = &mut self.state {
            let _ = sender.send(OcrWorkerMessage::Shutdown);
            if let Some(join) = join.take() {
                let _ = join.join();
            }
        }
    }
}

impl AppState {
    async fn create_ocr_worker(
        &self,
        runtime_id: &str,
        profile_id: &str,
        model_id: &str,
    ) -> OcrRunWorker {
        let (runtime, profile, model_file) = {
            let state = self.inner.state.lock().await;
            (
                state
                    .runtime_variants
                    .iter()
                    .find(|item| item.runtime_id == runtime_id)
                    .cloned(),
                find_profile(profile_id),
                find_model(model_id).map(|entry| entry.model_file),
            )
        };
        let Some(runtime) = runtime.filter(|item| item.selectable) else {
            return OcrRunWorker::fallback("runtime is not selectable");
        };
        let Some(profile) = profile else {
            return OcrRunWorker::fallback("OCR profile was not found");
        };
        let Some(model_file) = model_file else {
            return OcrRunWorker::fallback("model was not found");
        };
        let paths = crate::ocr::runtime_paths(&self.inner.config.app_root, &runtime, model_file);
        if !paths.model.is_file() || !paths.mmproj.is_file() || !paths.ffi_library.is_file() {
            return OcrRunWorker::fallback("native OCR assets are not installed");
        }
        OcrRunWorker::spawn(paths, profile, self.inner.hub.clone())
    }

    async fn runtime_stream_metadata(&self, runtime_id: &str) -> (String, String) {
        let state = self.inner.state.lock().await;
        state
            .runtime_variants
            .iter()
            .find(|item| item.runtime_id == runtime_id).map_or_else(|| (runtime_id.to_string(), String::new()), |item| (item.platform.clone(), item.accelerator.clone()))
    }
}

fn run_worker_loop(
    engine: &mut crate::ocr::UnlimitedOcrFfiEngine,
    receiver: mpsc::Receiver<OcrWorkerMessage>,
    max_tokens: i32,
    hub: &RealtimeHub,
) {
    for message in receiver {
        match message {
            OcrWorkerMessage::Recognize(request) => {
                let result =
                    recognize_on_worker(engine, request.image_path.as_path(), &request, max_tokens, hub);
                let _ = request.response.send(result);
            }
            OcrWorkerMessage::Shutdown => break,
        }
    }
}

fn recognize_on_worker(
    engine: &mut crate::ocr::UnlimitedOcrFfiEngine,
    image_path: &Path,
    request: &OcrWorkerRequest,
    max_tokens: i32,
    hub: &RealtimeHub,
) -> crate::ocr::OcrResult {
    let mut telemetry = OcrStreamTelemetry::new();
    let result = engine.recognize_image(image_path, max_tokens, |event| {
        if let crate::ocr::OcrEvent::Token { text, index } = event {
            publish_token_events(hub, &request.context, &mut telemetry, &text, index);
        }
    });
    finish_token_events(hub, &request.context, &mut telemetry);
    result
}

fn ocr_failure(message: impl Into<String>) -> crate::ocr::OcrResult {
    crate::ocr::OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message.into()),
    }
}

#[cfg(test)]
mod ocr_worker_tests {
    use super::*;

    #[test]
    fn fallback_worker_returns_failure_result() {
        let worker = OcrRunWorker::fallback("native OCR assets are not installed");
        let result = worker.recognize(Path::new("missing.png"), stream_context());

        assert!(!result.ok);
        assert_eq!(
            result.error.as_deref(),
            Some("native OCR assets are not installed")
        );
    }

    fn stream_context() -> OcrStreamContext {
        OcrStreamContext {
            run_id: "run-a".to_string(),
            file_hash: "file-a".to_string(),
            page_no: 1,
            engine_id: ENGINE_ID.to_string(),
            profile_id: "experimental-exact-prefill-q4".to_string(),
            model_id: "unlimited-ocr-q4-k-m".to_string(),
            runtime_id: "windows-x86_64-cuda13".to_string(),
            runtime_platform: "windows-x86_64-cuda13".to_string(),
            accelerator: "cuda".to_string(),
        }
    }
}
