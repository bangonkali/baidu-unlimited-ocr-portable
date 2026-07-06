fn waterfall_run_ids(rows: &[WaterfallDraft]) -> BTreeSet<String> {
    rows.iter()
        .filter_map(waterfall_row_run_id)
        .map(str::to_string)
        .collect()
}

fn ocr_run_ids(rows: &[WaterfallDraft]) -> BTreeSet<String> {
    rows.iter()
        .filter(|row| is_ocr_pipeline_row(row))
        .filter_map(waterfall_row_run_id)
        .map(str::to_string)
        .collect()
}

fn ocr_file_groups(rows: &[WaterfallDraft]) -> Vec<(String, String, Option<String>)> {
    let mut groups = BTreeMap::<(String, String), Option<String>>::new();
    for row in rows.iter().filter(|row| is_ocr_pipeline_row(row)) {
        let Some(run_id) = waterfall_row_run_id(row) else {
            continue;
        };
        let Some(file_hash) = row.file_hash.as_ref() else {
            continue;
        };
        let entry = groups
            .entry((run_id.to_string(), file_hash.clone()))
            .or_insert_with(|| row.filename.clone());
        if entry.is_none() && row.filename.is_some() {
            entry.clone_from(&row.filename);
        }
    }
    groups
        .into_iter()
        .map(|((run_id, file_hash), filename)| (run_id, file_hash, filename))
        .collect()
}

fn waterfall_row_run_id(row: &WaterfallDraft) -> Option<&str> {
    row.run_id
        .as_deref()
        .or_else(|| (!row.trace_id.is_empty()).then_some(row.trace_id.as_str()))
}

fn row_belongs_to_run(row: &WaterfallDraft, run_id: &str) -> bool {
    waterfall_row_run_id(row) == Some(run_id)
}

fn is_ocr_pipeline_row(row: &WaterfallDraft) -> bool {
    matches!(row.pipeline_step.as_str(), "ocr" | "render")
}

fn is_synthetic_waterfall_group(row: &WaterfallDraft) -> bool {
    matches!(
        row.row_source.as_str(),
        "run" | "task_group" | "file_group"
    )
}

fn aggregate_group_status<'a>(rows: impl Iterator<Item = &'a WaterfallDraft>) -> String {
    let mut saw_child = false;
    let mut saw_active = false;
    for row in rows {
        saw_child = true;
        match row.status.as_str() {
            "failed" | "error" => return "failed".to_string(),
            "queued" | "planned" | "running" => saw_active = true,
            _ => {}
        }
    }
    if saw_active {
        "running".to_string()
    } else if saw_child {
        "completed".to_string()
    } else {
        "planned".to_string()
    }
}

fn run_root_row_id(run_id: &str) -> String {
    format!("run:{run_id}")
}

fn ocr_task_group_row_id(run_id: &str) -> String {
    format!("group:{run_id}:ocr")
}

fn ocr_file_group_row_id(run_id: &str, file_hash: &str) -> String {
    format!("file-group:{run_id}:{file_hash}:ocr")
}

fn short_waterfall_id(value: &str) -> String {
    if value.len() > 8 {
        value[..8].to_string()
    } else {
        value.to_string()
    }
}
