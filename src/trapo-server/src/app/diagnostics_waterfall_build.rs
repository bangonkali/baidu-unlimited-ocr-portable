#[derive(Clone, Debug)]
struct WaterfallDraft {
    attributes: Value,
    category: String,
    child_count: u32,
    depth: u32,
    duration_ms: f64,
    end_ms: Option<f64>,
    ended_at: Option<String>,
    error_message: Option<String>,
    error_type: Option<String>,
    file_hash: Option<String>,
    filename: Option<String>,
    label: String,
    page_no: Option<u32>,
    parent_row_id: Option<String>,
    pipeline_step: String,
    row_source: String,
    row_id: String,
    run_id: Option<String>,
    sort_index: u32,
    span_id: Option<String>,
    span_kind: String,
    start_ms: Option<f64>,
    started_at: Option<String>,
    status: String,
    task_id: Option<String>,
    trace_id: String,
    visual_end_ms: Option<f64>,
    visual_duration_ms: f64,
    visual_start_ms: Option<f64>,
    work_unit_id: Option<String>,
}

fn build_diagnostic_waterfall(
    run_id: Option<String>,
    spans: Vec<DiagnosticSpanRow>,
    work_units: Vec<DiagnosticWorkUnitRow>,
    pipeline_tasks: Vec<PipelineTaskRow>,
) -> DiagnosticWaterfallPayload {
    let now_ms = timestamp_millis_f64(Utc::now().timestamp_millis());
    let work_unit_matches = matched_work_units(&spans, &work_units);
    let mut rows = Vec::new();
    rows.extend(
        pipeline_tasks
            .into_iter()
            .map(|task| task_waterfall_row(task, now_ms)),
    );
    rows.extend(
        work_units
            .into_iter()
            .filter(|unit| !work_unit_matches.contains_key(&unit.work_unit_id))
            .map(|unit| work_unit_waterfall_row(unit, now_ms)),
    );
    rows.extend(
        spans
            .into_iter()
            .map(|span| span_waterfall_row(span, now_ms, &work_unit_matches)),
    );
    fold_duplicate_task_spans(&mut rows);
    add_synthetic_waterfall_groups(&mut rows);
    normalize_waterfall_rows(&mut rows);
    let summary = waterfall_summary(run_id, &rows);
    DiagnosticWaterfallPayload {
        summary,
        rows: rows.into_iter().map(WaterfallDraft::into_record).collect(),
    }
}

fn task_waterfall_row(task: PipelineTaskRow, now_ms: f64) -> WaterfallDraft {
    let start = parse_timestamp_ms(task.started_at.as_ref().unwrap_or(&task.queued_at), false);
    let end = task
        .finished_at
        .as_deref()
        .and_then(|value| parse_timestamp_ms(value, false))
        .or_else(|| active_status_end_ms(&task.status, now_ms));
    WaterfallDraft {
        attributes: json!({"params": task.params, "result": task.result, "runner_id": task.runner_id}),
        category: "pipeline_task".to_string(),
        child_count: 0,
        depth: 0,
        duration_ms: duration_from_bounds(start, end),
        end_ms: end,
        ended_at: task.finished_at.clone(),
        error_message: task.error.clone(),
        error_type: task.error.as_ref().map(|_| "PipelineTask".to_string()),
        file_hash: None,
        filename: None,
        label: format!("{} - {}", task.task_kind, task.status),
        page_no: None,
        parent_row_id: None,
        pipeline_step: task.task_kind.clone(),
        row_source: "pipeline_task".to_string(),
        row_id: format!("task:{}", task.task_id),
        run_id: task.origin_run_id.clone(),
        sort_index: 0,
        span_id: None,
        span_kind: "task".to_string(),
        start_ms: start,
        started_at: task.started_at.or(Some(task.queued_at)),
        status: task.status,
        task_id: Some(task.task_id),
        trace_id: task.origin_run_id.unwrap_or_else(|| "unscoped".to_string()),
        visual_end_ms: end,
        visual_duration_ms: duration_from_bounds(start, end),
        visual_start_ms: start,
        work_unit_id: None,
    }
}

fn work_unit_waterfall_row(unit: DiagnosticWorkUnitRow, now_ms: f64) -> WaterfallDraft {
    let queued_start = parse_timestamp_ms(&unit.queued_at, false);
    let start = unit
        .started_at
        .as_deref()
        .and_then(|value| parse_timestamp_ms(value, false))
        .or(queued_start);
    let end = unit
        .finished_at
        .as_deref()
        .and_then(|value| parse_timestamp_ms(value, false))
        .or_else(|| active_status_end_ms(&unit.status, now_ms));
    let label = unit
        .page_no
        .map_or_else(|| unit.phase.clone(), |page| format!("{} page {page}", unit.phase));
    WaterfallDraft {
        attributes: json!({"result": unit.result, "metadata": unit.metadata}),
        category: "work_unit".to_string(),
        child_count: 0,
        depth: 0,
        duration_ms: unit.duration_ms.unwrap_or_else(|| duration_from_bounds(start, end)),
        end_ms: end,
        ended_at: unit.finished_at,
        error_message: unit.error.clone(),
        error_type: unit.error.as_ref().map(|_| "WorkUnit".to_string()),
        file_hash: unit.file_hash,
        filename: unit.filename.or(unit.source_path),
        label,
        page_no: unit.page_no,
        parent_row_id: None,
        pipeline_step: unit.phase,
        row_source: "work_unit".to_string(),
        row_id: format!("work:{}", unit.work_unit_id),
        run_id: Some(unit.run_id.clone()),
        sort_index: 0,
        span_id: None,
        span_kind: "work_unit".to_string(),
        start_ms: start,
        started_at: unit.started_at,
        status: unit.status,
        task_id: None,
        trace_id: unit.run_id,
        visual_end_ms: end,
        visual_duration_ms: duration_from_bounds(start, end),
        visual_start_ms: start,
        work_unit_id: Some(unit.work_unit_id),
    }
}

fn span_waterfall_row(
    span: DiagnosticSpanRow,
    now_ms: f64,
    work_unit_matches: &HashMap<String, DiagnosticWorkUnitRow>,
) -> WaterfallDraft {
    let start = parse_timestamp_ms(&span.started_at, true);
    let end = parse_timestamp_ms(&span.ended_at, false)
        .or_else(|| active_status_end_ms(&span.status, now_ms));
    let parent_row_id = span
        .parent_span_id
        .as_ref()
        .map(|parent| format!("span:{parent}"))
        .or_else(|| span.task_id.as_ref().map(|task| format!("task:{task}")));
    let label = span
        .page_no
        .map_or_else(|| span.name.clone(), |page| format!("{} page {page}", span.name));
    let matched_unit = matching_work_unit_for_span(&span, work_unit_matches);
    let attributes = matched_unit.map_or_else(
        || span.attributes.clone(),
        |unit| span_attributes_with_work_unit(&span, unit),
    );
    let work_unit_id = matched_unit
        .map(|unit| unit.work_unit_id.clone())
        .or_else(|| span.work_unit_id.clone());
    let filename = matched_unit.and_then(|unit| {
        unit.filename
            .clone()
            .or_else(|| unit.source_path.clone())
    }).or_else(|| span.filename.clone());
    WaterfallDraft {
        attributes,
        category: span.category,
        child_count: 0,
        depth: 0,
        duration_ms: span.duration_ms,
        end_ms: end,
        ended_at: Some(span.ended_at),
        error_message: span.error_message,
        error_type: span.error_type,
        file_hash: span.file_hash,
        filename,
        label,
        page_no: span.page_no,
        parent_row_id,
        pipeline_step: span.pipeline_step,
        row_source: "diagnostic_span".to_string(),
        row_id: format!("span:{}", span.span_id),
        run_id: span.run_id,
        sort_index: 0,
        span_id: Some(span.span_id),
        span_kind: span.span_kind,
        start_ms: start,
        started_at: Some(span.started_at),
        status: span.status,
        task_id: span.task_id,
        trace_id: span.trace_id,
        visual_end_ms: end,
        visual_duration_ms: duration_from_bounds(start, end),
        visual_start_ms: start,
        work_unit_id,
    }
}
