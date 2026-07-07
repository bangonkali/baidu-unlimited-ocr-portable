impl OcrRunWorker {
    fn adapter(reason: impl Into<String>) -> Self {
        Self {
            state: OcrRunWorkerState::Adapter {
                reason: reason.into(),
            },
        }
    }

    const fn is_ready(&self) -> bool {
        matches!(self.state, OcrRunWorkerState::Ready { .. })
    }
}

impl AppState {
    async fn create_adapter_ocr_worker(&self, engine_id: &str, runtime_id: &str) -> OcrRunWorker {
        let runtime_id = self.adapter_runtime_id(runtime_id).await;
        let worker = self
            .create_unlimited_ocr_worker(&runtime_id, DEFAULT_PROFILE_ID, DEFAULT_MODEL_ID)
            .await;
        if worker.is_ready() {
            return worker;
        }
        OcrRunWorker::adapter(adapter_worker_reason(engine_id, &worker.fallback_reason()))
    }

    async fn adapter_runtime_id(&self, runtime_id: &str) -> String {
        if !runtime_id.is_empty() {
            return runtime_id.to_string();
        }
        self.inner.state.lock().await.selected_runtime_id.clone()
    }
}

fn ocr_adapter_result(
    image_path: &Path,
    context: &OcrStreamContext,
    reason: &str,
) -> crate::ocr::OcrResult {
    crate::ocr::OcrResult {
        ok: true,
        text: compatibility_adapter_text(image_path, context, reason),
        error: None,
    }
}

fn compatibility_adapter_text(image_path: &Path, context: &OcrStreamContext, reason: &str) -> String {
    let filename = image_path.file_name().map_or_else(
        || image_path.to_string_lossy().to_string(),
        |value| value.to_string_lossy().to_string(),
    );
    format!(
        "Compatibility adapter output for {filename} using {}. Native runner status: {reason}.",
        context.engine_id
    )
}

fn adapter_worker_reason(engine_id: &str, fallback_reason: &str) -> String {
    format!("{engine_id} compatibility adapter active; {fallback_reason}")
}
