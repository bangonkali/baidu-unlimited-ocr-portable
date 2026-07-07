fn add_synthetic_waterfall_groups(rows: &mut Vec<WaterfallDraft>) {
    let snapshot = rows.clone();
    add_run_roots(rows, &snapshot);
    add_ocr_groups(rows, &snapshot);
    add_rag_file_groups(rows, &snapshot);
    reparent_waterfall_roots(rows);
}

fn fold_duplicate_task_spans(rows: &mut Vec<WaterfallDraft>) {
    let task_rows = rows
        .iter()
        .filter(|row| row.row_source == "pipeline_task")
        .filter_map(|row| row.task_id.as_ref().map(|task_id| (task_id.clone(), row.row_id.clone())))
        .collect::<HashMap<_, _>>();
    let duplicate_parents = rows
        .iter()
        .filter(|row| row.row_source == "diagnostic_span" && row.span_kind == "task")
        .filter_map(|row| {
            let task_parent = task_rows.get(row.task_id.as_ref()?)?;
            Some((row.row_id.clone(), task_parent.clone()))
        })
        .collect::<HashMap<_, _>>();
    if duplicate_parents.is_empty() {
        return;
    }
    for row in rows.iter_mut() {
        if let Some(parent) = row
            .parent_row_id
            .as_ref()
            .and_then(|parent| duplicate_parents.get(parent))
        {
            row.parent_row_id = Some(parent.clone());
        }
    }
    rows.retain(|row| !duplicate_parents.contains_key(&row.row_id));
}

fn add_run_roots(rows: &mut Vec<WaterfallDraft>, snapshot: &[WaterfallDraft]) {
    for run_id in waterfall_run_ids(snapshot) {
        rows.push(synthetic_group_row(SyntheticGroupInput {
            attributes: json!({"run_id": run_id}),
            category: "run".to_string(),
            file_hash: None,
            filename: None,
            label: format!("Run {}", short_waterfall_id(&run_id)),
            page_no: None,
            parent_row_id: None,
            pipeline_step: "run".to_string(),
            row_id: run_root_row_id(&run_id),
            row_source: "run".to_string(),
            run_id: Some(run_id.clone()),
            span_kind: "run".to_string(),
            status: aggregate_group_status(snapshot.iter().filter(|row| row_belongs_to_run(row, &run_id))),
            trace_id: run_id,
        }));
    }
}

fn add_ocr_groups(rows: &mut Vec<WaterfallDraft>, snapshot: &[WaterfallDraft]) {
    for run_id in ocr_run_ids(snapshot) {
        rows.push(synthetic_group_row(SyntheticGroupInput {
            attributes: json!({"pipeline_step": "ocr"}),
            category: "task_group".to_string(),
            file_hash: None,
            filename: None,
            label: "OCR".to_string(),
            page_no: None,
            parent_row_id: Some(run_root_row_id(&run_id)),
            pipeline_step: "ocr".to_string(),
            row_id: ocr_task_group_row_id(&run_id),
            row_source: "task_group".to_string(),
            run_id: Some(run_id.clone()),
            span_kind: "task_group".to_string(),
            status: aggregate_group_status(snapshot.iter().filter(|row| {
                row_belongs_to_run(row, &run_id) && is_ocr_pipeline_row(row)
            })),
            trace_id: run_id,
        }));
    }
    for (run_id, file_hash, filename) in ocr_file_groups(snapshot) {
        rows.push(synthetic_group_row(SyntheticGroupInput {
            attributes: json!({"file_hash": file_hash}),
            category: "file_group".to_string(),
            file_hash: Some(file_hash.clone()),
            filename: filename.clone(),
            label: filename.unwrap_or_else(|| short_waterfall_id(&file_hash)),
            page_no: None,
            parent_row_id: Some(ocr_task_group_row_id(&run_id)),
            pipeline_step: "ocr".to_string(),
            row_id: ocr_file_group_row_id(&run_id, &file_hash),
            row_source: "file_group".to_string(),
            run_id: Some(run_id.clone()),
            span_kind: "file_group".to_string(),
            status: aggregate_group_status(snapshot.iter().filter(|row| {
                row_belongs_to_run(row, &run_id)
                    && row.file_hash.as_deref() == Some(file_hash.as_str())
                    && is_ocr_pipeline_row(row)
            })),
            trace_id: run_id,
        }));
    }
}

fn add_rag_file_groups(rows: &mut Vec<WaterfallDraft>, snapshot: &[WaterfallDraft]) {
    for (run_id, task_id, pipeline_step, file_hash, filename) in rag_file_groups(snapshot) {
        rows.push(synthetic_group_row(SyntheticGroupInput {
            attributes: json!({"file_hash": file_hash.as_str(), "pipeline_step": pipeline_step.as_str()}),
            category: "file_group".to_string(),
            file_hash: Some(file_hash.clone()),
            filename: filename.clone(),
            label: filename.unwrap_or_else(|| short_waterfall_id(&file_hash)),
            page_no: None,
            parent_row_id: Some(format!("task:{task_id}")),
            pipeline_step: pipeline_step.clone(),
            row_id: rag_file_group_row_id(&task_id, &file_hash, &pipeline_step),
            row_source: "file_group".to_string(),
            run_id: Some(run_id.clone()),
            span_kind: "file_group".to_string(),
            status: aggregate_group_status(snapshot.iter().filter(|row| {
                row_belongs_to_run(row, &run_id)
                    && row.task_id.as_deref() == Some(task_id.as_str())
                    && row.file_hash.as_deref() == Some(file_hash.as_str())
                    && row.pipeline_step == pipeline_step
            })),
            trace_id: run_id,
        }));
    }
}

fn reparent_waterfall_roots(rows: &mut [WaterfallDraft]) {
    for row in rows.iter_mut() {
        if is_synthetic_waterfall_group(row) {
            continue;
        }
        if maybe_reparent_rag_file_row(row) {
            continue;
        }
        if row.parent_row_id.is_some() {
            continue;
        }
        let Some(run_id) = waterfall_row_run_id(row).map(str::to_string) else {
            continue;
        };
        row.parent_row_id = Some(if is_ocr_pipeline_row(row) {
            row.file_hash.as_ref().map_or_else(
                || ocr_task_group_row_id(&run_id),
                |file_hash| ocr_file_group_row_id(&run_id, file_hash),
            )
        } else {
            run_root_row_id(&run_id)
        });
    }
}

fn maybe_reparent_rag_file_row(row: &mut WaterfallDraft) -> bool {
    if !is_rag_pipeline_row(row) {
        return false;
    }
    let Some(task_id) = row.task_id.as_ref() else {
        return false;
    };
    let Some(file_hash) = row.file_hash.as_ref() else {
        return false;
    };
    let task_parent = format!("task:{task_id}");
    if row.parent_row_id.as_deref() != Some(task_parent.as_str()) {
        return false;
    }
    row.parent_row_id = Some(rag_file_group_row_id(
        task_id,
        file_hash,
        &row.pipeline_step,
    ));
    true
}

fn synthetic_group_row(input: SyntheticGroupInput) -> WaterfallDraft {
    WaterfallDraft {
        attributes: input.attributes,
        activity_kind: "internal".to_string(),
        category: input.category,
        child_count: 0,
        depth: 0,
        duration_ms: 0.0,
        end_ms: None,
        ended_at: None,
        error_message: None,
        error_type: None,
        file_hash: input.file_hash,
        filename: input.filename,
        label: input.label,
        page_no: input.page_no,
        parent_row_id: input.parent_row_id,
        pipeline_step: input.pipeline_step,
        row_id: input.row_id,
        row_source: input.row_source,
        run_id: input.run_id,
        sort_index: 0,
        span_id: None,
        span_kind: input.span_kind,
        start_ms: None,
        started_at: None,
        status: input.status,
        status_code: "unset".to_string(),
        status_message: None,
        task_id: None,
        trace_id: input.trace_id,
        visual_end_ms: None,
        visual_duration_ms: 0.0,
        visual_start_ms: None,
        work_unit_id: None,
    }
}

struct SyntheticGroupInput {
    attributes: Value,
    category: String,
    file_hash: Option<String>,
    filename: Option<String>,
    label: String,
    page_no: Option<u32>,
    parent_row_id: Option<String>,
    pipeline_step: String,
    row_id: String,
    row_source: String,
    run_id: Option<String>,
    span_kind: String,
    status: String,
    trace_id: String,
}
