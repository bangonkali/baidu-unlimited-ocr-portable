fn run_worker_loop(
    mut backend: OcrWorkerBackend,
    receiver: mpsc::Receiver<OcrWorkerMessage>,
    max_tokens: i32,
    hub: &RealtimeHub,
    annotation_identities: &AnnotationIdentityRuntime,
) {
    for message in receiver {
        match message {
            OcrWorkerMessage::Recognize(request) => {
                let result = recognize_on_worker(
                    &mut backend,
                    request.image_path.as_path(),
                    &request,
                    max_tokens,
                    hub,
                    annotation_identities,
                );
                let _ = request.response.send(result);
            }
            OcrWorkerMessage::Shutdown => break,
        }
    }
}

fn recognize_on_worker(
    backend: &mut OcrWorkerBackend,
    image_path: &Path,
    request: &OcrWorkerRequest,
    max_tokens: i32,
    hub: &RealtimeHub,
    annotation_identities: &AnnotationIdentityRuntime,
) -> crate::ocr::OcrResult {
    match backend {
        OcrWorkerBackend::Unlimited(engine) => {
            let mut telemetry = OcrStreamTelemetry::new();
            let result = engine.recognize_image(image_path, max_tokens, |event| {
                if let crate::ocr::OcrEvent::Token { text, index } = event {
                    publish_token_events(
                        hub,
                        Some(annotation_identities),
                        &request.context,
                        &mut telemetry,
                        &text,
                        index,
                    );
                }
            });
            finish_token_events(hub, &request.context, &mut telemetry);
            result
        }
        OcrWorkerBackend::Adapter(runner) => runner.recognize(image_path),
    }
}

fn ocr_failure(message: impl Into<String>) -> crate::ocr::OcrResult {
    crate::ocr::OcrResult {
        ok: false,
        text: String::new(),
        error: Some(message.into()),
    }
}
