impl AppState {
    fn render_document_file(
        &self,
        file_hash: &str,
        document_path: &Path,
    ) -> Result<Vec<RenderedPage>> {
        if is_pdf(document_path) {
            self.log_info(
                "pdfium",
                format!(
                    "rendering {} at {PDF_DPI} DPI with PDFium",
                    document_path.display()
                ),
            );
            self.inner.renderer.render_pdf(file_hash, document_path)
        } else {
            PdfRenderer::image_page(document_path).map(|page| vec![page])
        }
    }

    async fn finish_render_diagnostics(
        &self,
        run_id: &str,
        file_hash: &str,
        work_unit: &DiagnosticWorkUnitHandle,
        span: DiagnosticSpanScope,
        rendered: &[RenderedPage],
    ) {
        self.record_span(
            span,
            SpanFinish {
                run_id,
                task_id: None,
                work_unit_id: Some(&work_unit.id),
                parent_span_id: None,
                file_hash: Some(file_hash),
                page_no: None,
                name: "Render document",
                pipeline_step: "render",
                category: "file",
                engine: Some("pdfium"),
                status: "ok",
                error: None,
                attributes: json!({ "page_count": rendered.len() }),
            },
        );
        self.finish_diagnostic_work_unit(
            run_id,
            work_unit,
            "completed",
            None,
            json!({ "page_count": rendered.len() }),
        )
        .await;
    }

    async fn fail_render_diagnostics(
        &self,
        run_id: &str,
        file_hash: &str,
        work_unit: &DiagnosticWorkUnitHandle,
        span: DiagnosticSpanScope,
        error: &AppError,
    ) {
        let message = error.to_string();
        self.record_span(
            span,
            SpanFinish {
                run_id,
                task_id: None,
                work_unit_id: Some(&work_unit.id),
                parent_span_id: None,
                file_hash: Some(file_hash),
                page_no: None,
                name: "Render document",
                pipeline_step: "render",
                category: "file",
                engine: Some("pdfium"),
                status: "failed",
                error: Some(&message),
                attributes: json!({}),
            },
        );
        self.finish_diagnostic_work_unit(run_id, work_unit, "failed", Some(&message), json!({}))
            .await;
    }
}
