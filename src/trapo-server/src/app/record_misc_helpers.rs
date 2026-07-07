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
