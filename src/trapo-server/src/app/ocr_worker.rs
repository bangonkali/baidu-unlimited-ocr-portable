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

enum OcrWorkerBackend {
    Unlimited(crate::ocr::UnlimitedOcrFfiEngine),
    Adapter(Box<ocr_engines::EngineRunner>),
}

struct OcrWorkerRequest {
    image_path: PathBuf,
    context: OcrStreamContext,
    response: mpsc::Sender<crate::ocr::OcrResult>,
}

struct OcrRunContext<'a> {
    run_engine_id: &'a str,
    engine_kind: &'a str,
    engine_id: &'a str,
    profile_id: &'a str,
    model_id: &'a str,
    runtime_id: &'a str,
    runtime_platform: &'a str,
    accelerator: &'a str,
    worker: &'a OcrRunWorker,
    activity_context: ActivityContext,
}

struct PageWork<'a> {
    run_id: &'a str,
    work_unit_id: &'a str,
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

    fn spawn_unlimited(
        paths: crate::ocr::OcrRuntimePaths,
        profile: crate::types::OcrProfileRecord,
        hub: Arc<RealtimeHub>,
        annotation_identities: AnnotationIdentityRuntime,
    ) -> Self {
        let (ready_sender, ready_receiver) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();
        let max_tokens = u32_to_i32_saturating(profile.default_max_tokens);
        let join = match thread::Builder::new()
            .name("trapo-ocr-worker".to_string())
            .spawn(move || {
                let engine = match crate::ocr::UnlimitedOcrFfiEngine::load(&paths, &profile) {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = ready_sender.send(Err(error.to_string()));
                        return;
                    }
                };
                let _ = ready_sender.send(Ok(()));
                run_worker_loop(
                    OcrWorkerBackend::Unlimited(engine),
                    receiver,
                    max_tokens,
                    &hub,
                    &annotation_identities,
                );
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

    fn spawn_adapter(
        runner: ocr_engines::EngineRunner,
        hub: Arc<RealtimeHub>,
        annotation_identities: AnnotationIdentityRuntime,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let join = match thread::Builder::new()
            .name(format!("trapo-{}-worker", runner.engine_id()))
            .spawn(move || {
                run_worker_loop(
                    OcrWorkerBackend::Adapter(Box::new(runner)),
                    receiver,
                    0,
                    &hub,
                    &annotation_identities,
                );
            }) {
            Ok(join) => join,
            Err(error) => {
                return Self::fallback(format!("could not start OCR adapter worker thread: {error}"));
            }
        };
        Self {
            state: OcrRunWorkerState::Ready {
                sender,
                join: Some(join),
            },
        }
    }

    fn recognize(&self, image_path: &Path, context: OcrStreamContext) -> crate::ocr::OcrResult {
        let OcrRunWorkerState::Ready { sender, .. } = &self.state else {
            return match &self.state {
                OcrRunWorkerState::Fallback(_) => ocr_failure(self.fallback_reason()),
                OcrRunWorkerState::Ready { .. } => unreachable!(),
            };
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
        engine_id: &str,
        runtime_id: &str,
        profile_id: &str,
        model_id: &str,
    ) -> OcrRunWorker {
        if engine_id != ENGINE_ID {
            return self
                .create_adapter_ocr_worker(engine_id, runtime_id, model_id)
                .await;
        }
        self.create_unlimited_ocr_worker(runtime_id, profile_id, model_id)
            .await
    }

    async fn create_unlimited_ocr_worker(
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
        OcrRunWorker::spawn_unlimited(
            paths,
            profile,
            self.inner.hub.clone(),
            self.inner.annotation_identities.clone(),
        )
    }

    async fn runtime_stream_metadata(&self, runtime_id: &str) -> (String, String) {
        let state = self.inner.state.lock().await;
        state
            .runtime_variants
            .iter()
            .find(|item| item.runtime_id == runtime_id).map_or_else(|| (runtime_id.to_string(), String::new()), |item| (item.platform.clone(), item.accelerator.clone()))
    }
}
