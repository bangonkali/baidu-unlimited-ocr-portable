impl AppState {
    async fn finish_page_diagnostics(
        &self,
        finish: PageDiagnosticFinish<'_>,
        span: DiagnosticSpanScope,
    ) {
        let (status, error) = match finish.result {
            Ok(()) => ("ok", None),
            Err(error) => ("failed", Some(error.to_string())),
        };
        let error_ref = error.as_deref();
        self.record_span(
            span,
            SpanFinish {
                run_id: finish.run_id,
                task_id: None,
                work_unit_id: Some(&finish.work_unit.id),
                parent_span_id: None,
                file_hash: Some(finish.file_hash),
                page_no: Some(finish.page.page_no),
                name: "OCR page",
                pipeline_step: "ocr",
                category: "page",
                engine: Some(finish.engine_id),
                status,
                error: error_ref,
                attributes: json!({
                    "image_path": finish.page.image_path.to_string_lossy().to_string()
                }),
            },
        );
        self.finish_diagnostic_work_unit(
            finish.run_id,
            finish.work_unit,
            if finish.result.is_ok() {
                "completed"
            } else {
                "failed"
            },
            error_ref,
            json!({ "page_no": finish.page.page_no }),
        )
        .await;
    }
}
