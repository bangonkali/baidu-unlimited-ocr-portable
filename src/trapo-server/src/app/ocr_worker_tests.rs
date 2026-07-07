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

    #[test]
    fn adapter_worker_returns_successful_compatibility_output() {
        let worker = OcrRunWorker::adapter(
            "tesseract-rs compatibility adapter active; native runtime not installed",
        );
        let mut context = stream_context();
        context.engine_id = "tesseract-rs".to_string();
        let result = worker.recognize(Path::new("page-1.png"), context);

        assert!(result.ok);
        assert!(worker.fallback_error().is_none());
        assert!(result.text.contains("Compatibility adapter output"));
        assert!(result.text.contains("tesseract-rs"));
    }

    fn stream_context() -> OcrStreamContext {
        OcrStreamContext {
            run_id: "run-a".to_string(),
            run_engine_id: "01980a3d-a4fc-7000-8000-000000000001".to_string(),
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
