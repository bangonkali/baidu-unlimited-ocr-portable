#[derive(Clone, Copy)]
struct PageDiagnosticFinish<'a> {
    run_id: &'a str,
    file_hash: &'a str,
    page: &'a RenderedPage,
    work_unit_id: &'a str,
    result: &'a Result<()>,
}

fn queued_page_state(page: &RenderedPage) -> PageState {
    PageState {
        page_no: page.page_no,
        image_path: page.image_path.clone(),
        width_px: page.width_px,
        height_px: page.height_px,
        render_dpi: PDF_DPI,
        status: "queued".to_string(),
        raw_text: String::new(),
        cleaned_text: String::new(),
        boxes: Vec::new(),
        spans: Vec::new(),
        error: None,
    }
}

fn page_diagnostic_metadata(page: &PageState) -> (u32, Value) {
    (
        page.page_no,
        json!({
            "width_px": page.width_px,
            "height_px": page.height_px,
            "render_dpi": page.render_dpi
        }),
    )
}

fn rendered_page_record(page: &StoredPage) -> Value {
    let mut record = json!({
        "file_hash": page.file_hash,
        "page_no": page.page_no,
        "status": page.status,
        "width_px": page.width_px,
        "height_px": page.height_px,
        "dpi": page.render_dpi,
        "preview_available": true,
    });
    if let Some(run_id) = &page.run_id {
        record["run_id"] = json!(run_id);
    }
    record
}

#[cfg(test)]
mod process_document_records_tests {
    use super::*;

    #[test]
    fn rendered_page_record_marks_preview_ready() {
        let record = rendered_page_record(&StoredPage {
            boxes: Vec::new(),
            cleaned_text: String::new(),
            error: None,
            file_hash: "file-a".to_string(),
            height_px: 200,
            page_no: 1,
            preview_path: Some("page-1.png".to_string()),
            raw_text: String::new(),
            render_dpi: 200,
            run_id: None,
            spans: Vec::new(),
            status: "queued".to_string(),
            width_px: 100,
        });

        assert_eq!(record["file_hash"], "file-a");
        assert_eq!(record["page_no"], 1);
        assert_eq!(record["preview_available"], true);
        assert_eq!(record["dpi"], 200);
    }
}
