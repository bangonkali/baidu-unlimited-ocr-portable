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
