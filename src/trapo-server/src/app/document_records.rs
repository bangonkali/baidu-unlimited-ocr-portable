fn document_from_file(file: &DiscoveredFile, root_path: &Path) -> DocumentState {
    let display_name = file
        .absolute_path
        .file_name().map_or_else(|| generic_path(&file.relative_path), |value| value.to_string_lossy().to_string());
    DocumentState {
        file_hash: stable_hash(file),
        display_name,
        extension: file
            .absolute_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase(),
        size_bytes: file.size_bytes,
        absolute_path: file.absolute_path.clone(),
        relative_path: file.relative_path.clone(),
        root_path: root_path.to_path_buf(),
        status: "queued".to_string(),
        page_count: 1,
        error: None,
        pages: Vec::new(),
    }
}

fn document_from_stored(stored: StoredDocument) -> DocumentState {
    DocumentState {
        file_hash: stored.file_hash,
        display_name: stored.display_name,
        extension: stored.extension,
        size_bytes: stored.size_bytes,
        absolute_path: PathBuf::from(stored.absolute_path),
        relative_path: PathBuf::from(stored.relative_path),
        root_path: PathBuf::from(stored.root_path),
        status: stored.status,
        page_count: stored.page_count,
        error: stored.error,
        pages: Vec::new(),
    }
}

fn page_from_stored(stored: StoredPage) -> PageState {
    PageState {
        page_no: stored.page_no,
        image_path: stored.preview_path.map(PathBuf::from).unwrap_or_default(),
        width_px: stored.width_px,
        height_px: stored.height_px,
        render_dpi: stored.render_dpi,
        status: stored.status,
        raw_text: stored.raw_text,
        cleaned_text: stored.cleaned_text,
        boxes: stored.boxes,
        spans: stored.spans,
        error: stored.error,
    }
}

fn stored_document(document: &DocumentState) -> StoredDocument {
    StoredDocument {
        file_hash: document.file_hash.clone(),
        display_name: document.display_name.clone(),
        extension: document.extension.clone(),
        size_bytes: document.size_bytes,
        page_count: document.page_count,
        status: document.status.clone(),
        error: document.error.clone(),
        root_path: document.root_path.to_string_lossy().to_string(),
        absolute_path: document.absolute_path.to_string_lossy().to_string(),
        relative_path: generic_path(&document.relative_path),
    }
}

fn stored_page(file_hash: &str, page: &PageState) -> StoredPage {
    StoredPage {
        file_hash: file_hash.to_string(),
        page_no: page.page_no,
        width_px: page.width_px,
        height_px: page.height_px,
        render_dpi: page.render_dpi,
        status: page.status.clone(),
        error: page.error.clone(),
        preview_path: Some(page.image_path.to_string_lossy().to_string()),
        cleaned_text: page.cleaned_text.clone(),
        raw_text: page.raw_text.clone(),
        boxes: page.boxes.clone(),
        spans: page.spans.clone(),
    }
}

fn stored_run(run: &RunState) -> StoredRun {
    StoredRun {
        run_id: run.run_id.clone(),
        root_path: run.root_path.clone(),
        status: run.status.clone(),
        profile_id: run.profile_id.clone(),
        engine_id: run.engine_id.clone(),
        model_id: run.model_id.clone(),
        runtime_id: run.runtime_id.clone(),
        queued_files: run.queued_files,
        processed_pages: run.processed_pages,
        total_pages: run.total_pages,
        error: run.error.clone(),
    }
}

fn run_from_stored(stored: StoredRun) -> RunState {
    RunState {
        run_id: stored.run_id,
        root_path: stored.root_path,
        status: stored.status,
        queued_files: stored.queued_files,
        processed_pages: stored.processed_pages,
        total_pages: stored.total_pages,
        current_page: None,
        profile_id: stored.profile_id,
        engine_id: stored.engine_id,
        model_id: if stored.model_id.is_empty() {
            DEFAULT_MODEL_ID.to_string()
        } else {
            stored.model_id
        },
        runtime_id: stored.runtime_id,
        error: stored.error,
        cancel_requested: false,
        file_hashes: Vec::new(),
    }
}

fn run_record(run: &RunState) -> IngestRunRecord {
    IngestRunRecord {
        run_id: run.run_id.clone(),
        root_path: run.root_path.clone(),
        status: run.status.clone(),
        file_hashes: run.file_hashes.clone(),
        queued_files: run.queued_files,
        processed_pages: run.processed_pages,
        total_pages: run.total_pages,
        current_page: run.current_page,
        progress_percent: percent(run.processed_pages, run.total_pages),
        profile_id: run.profile_id.clone(),
        engine_id: run.engine_id.clone(),
        model_id: run.model_id.clone(),
        runtime_id: run.runtime_id.clone(),
        error: run.error.clone(),
    }
}

fn document_summary(document: &DocumentState) -> DocumentSummary {
    let processed_pages = document
        .pages
        .iter()
        .filter(|page| page.status == "completed")
        .count();
    let current_page = document
        .pages
        .iter()
        .find(|page| page.status == "running")
        .map(|page| page.page_no);
    DocumentSummary {
        file_hash: document.file_hash.clone(),
        display_name: document.display_name.clone(),
        relative_path: generic_path(&document.relative_path),
        status: document.status.clone(),
        page_count: document.page_count,
        processed_pages: usize_to_u32_saturating(processed_pages),
        total_pages: document.page_count,
        current_page,
        progress_percent: percent(usize_to_u32_saturating(processed_pages), document.page_count),
        regions: document
            .pages
            .iter()
            .fold(0_u32, |total, page| {
                total.saturating_add(usize_to_u32_saturating(page.boxes.len()))
            }),
        error: document.error.clone(),
    }
}

fn started_page_text_records(document: &DocumentState) -> Vec<PageTextRecord> {
    document
        .pages
        .iter()
        .filter(|page| page.status != "queued")
        .map(page_text_record)
        .collect()
}

fn page_text_record(page: &PageState) -> PageTextRecord {
    PageTextRecord {
        page_no: page.page_no,
        text: page.cleaned_text.clone(),
        spans: page.spans.clone(),
    }
}

fn document_detail(document: &DocumentState) -> DocumentDetail {
    let summary = document_summary(document);
    DocumentDetail {
        file_hash: summary.file_hash,
        display_name: summary.display_name,
        relative_path: summary.relative_path,
        absolute_path: document.absolute_path.to_string_lossy().to_string(),
        status: summary.status,
        page_count: summary.page_count,
        processed_pages: summary.processed_pages,
        total_pages: summary.total_pages,
        current_page: summary.current_page,
        progress_percent: summary.progress_percent,
        regions: summary.regions,
        error: summary.error,
    }
}

fn metrics_tree(rows: Vec<OcrPageMetrics>) -> OcrMetricsTreePayload {
    OcrMetricsTreePayload {
        roots: rows
            .into_iter()
            .map(|row| OcrMetricsTreeNode {
                id: format!("{}:{}:{}", row.run_id, row.file_hash, row.page_no),
                label: format!("{} page {}", row.file_hash, row.page_no),
                kind: "page".to_string(),
                status: row.status,
                token_count: row.token_count,
                avg_tps: row.avg_tps,
                elapsed_ms: row.elapsed_ms,
                children: Vec::new(),
            })
            .collect(),
    }
}

fn fallback_text(image_path: &Path, reason: &str) -> String {
    format!(
        "OCR was not executed for {} because {reason}.",
        image_path
            .file_name().map_or_else(|| image_path.to_string_lossy().to_string(), |value| value.to_string_lossy().to_string())
    )
}

fn percent(done: u32, total: u32) -> f64 {
    if total == 0 {
        0.0
    } else {
        f64::from(done) / f64::from(total) * 100.0
    }
}

fn now_id() -> String {
    new_persistence_id()
}

fn hf_token() -> Option<String> {
    std::env::var("HF_TOKEN")
        .ok()
        .or_else(|| std::env::var("HUGGING_FACE_HUB_TOKEN").ok())
        .filter(|value| !value.is_empty())
}
