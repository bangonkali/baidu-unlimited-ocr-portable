fn normalize_waterfall_rows(rows: &mut [WaterfallDraft]) {
    let by_id = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.row_id.clone(), index))
        .collect::<HashMap<_, _>>();
    for row in rows.iter_mut() {
        if row
            .parent_row_id
            .as_ref()
            .is_some_and(|parent| !by_id.contains_key(parent))
        {
            row.parent_row_id = None;
        }
    }
    let children = waterfall_children(rows, &by_id);
    for index in 0..rows.len() {
        let mut visiting = HashSet::new();
        let _ = compute_visual_bounds(index, rows, &children, &mut visiting);
    }
    for index in 0..rows.len() {
        rows[index].child_count = usize_to_u32_saturating(children[index].len());
        rows[index].depth = row_depth(index, rows, &by_id);
    }
    let order = waterfall_order(rows, &children);
    for (sort_index, row_index) in order.into_iter().enumerate() {
        rows[row_index].sort_index = usize_to_u32_saturating(sort_index);
    }
    rows.sort_by_key(|row| row.sort_index);
}

fn waterfall_children(
    rows: &[WaterfallDraft],
    by_id: &HashMap<String, usize>,
) -> Vec<Vec<usize>> {
    let mut children = vec![Vec::new(); rows.len()];
    for (index, row) in rows.iter().enumerate() {
        if let Some(parent_index) = row.parent_row_id.as_ref().and_then(|id| by_id.get(id)) {
            children[*parent_index].push(index);
        }
    }
    children
}

fn compute_visual_bounds(
    index: usize,
    rows: &mut [WaterfallDraft],
    children: &[Vec<usize>],
    visiting: &mut HashSet<usize>,
) -> (Option<f64>, Option<f64>) {
    if !visiting.insert(index) {
        return (rows[index].visual_start_ms, rows[index].visual_end_ms);
    }
    let mut start = rows[index].start_ms;
    let mut end = rows[index].end_ms;
    for child in &children[index] {
        let (child_start, child_end) = compute_visual_bounds(*child, rows, children, visiting);
        start = min_optional_ms(start, child_start);
        end = max_optional_ms(end, child_end);
    }
    rows[index].visual_start_ms = start;
    rows[index].visual_end_ms = end;
    rows[index].visual_duration_ms = duration_from_bounds(start, end);
    visiting.remove(&index);
    (start, end)
}

fn row_depth(index: usize, rows: &[WaterfallDraft], by_id: &HashMap<String, usize>) -> u32 {
    let mut depth = 0_u32;
    let mut current = index;
    let mut seen = HashSet::new();
    while let Some(parent) = rows[current].parent_row_id.as_ref() {
        let Some(parent_index) = by_id.get(parent) else {
            break;
        };
        if !seen.insert(*parent_index) {
            break;
        }
        depth = depth.saturating_add(1);
        current = *parent_index;
    }
    depth
}

fn waterfall_order(rows: &[WaterfallDraft], children: &[Vec<usize>]) -> Vec<usize> {
    let mut roots = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.parent_row_id.is_none().then_some(index))
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| row_time_key(&rows[*left]).total_cmp(&row_time_key(&rows[*right])));
    let mut order = Vec::with_capacity(rows.len());
    for root in roots {
        push_waterfall_order(root, rows, children, &mut order);
    }
    order
}

fn push_waterfall_order(
    index: usize,
    rows: &[WaterfallDraft],
    children: &[Vec<usize>],
    order: &mut Vec<usize>,
) {
    order.push(index);
    let mut sorted = children[index].clone();
    sorted.sort_by(|left, right| row_time_key(&rows[*left]).total_cmp(&row_time_key(&rows[*right])));
    for child in sorted {
        push_waterfall_order(child, rows, children, order);
    }
}

fn waterfall_summary(
    run_id: Option<String>,
    rows: &[WaterfallDraft],
) -> DiagnosticWaterfallSummary {
    let start_ms = rows
        .iter()
        .filter_map(|row| row.visual_start_ms)
        .min_by(f64::total_cmp);
    let end_ms = rows
        .iter()
        .filter_map(|row| row.visual_end_ms)
        .max_by(f64::total_cmp);
    let trace_count = rows
        .iter()
        .map(|row| row.trace_id.clone())
        .collect::<HashSet<_>>()
        .len();
    DiagnosticWaterfallSummary {
        run_id,
        row_count: usize_to_u32_saturating(rows.len()),
        trace_count: usize_to_u32_saturating(trace_count),
        error_count: usize_to_u32_saturating(
            rows.iter()
                .filter(|row| matches!(row.status.as_str(), "failed" | "error"))
                .count(),
        ),
        start_ms,
        end_ms,
        duration_ms: duration_from_bounds(start_ms, end_ms),
    }
}

impl WaterfallDraft {
    fn into_record(self) -> DiagnosticWaterfallRowRecord {
        DiagnosticWaterfallRowRecord {
            row_id: self.row_id,
            trace_id: self.trace_id,
            parent_row_id: self.parent_row_id,
            span_id: self.span_id,
            task_id: self.task_id,
            work_unit_id: self.work_unit_id,
            run_id: self.run_id,
            file_hash: self.file_hash,
            filename: self.filename,
            page_no: self.page_no,
            label: self.label,
            row_source: self.row_source,
            pipeline_step: self.pipeline_step,
            category: self.category,
            span_kind: self.span_kind,
            activity_kind: self.activity_kind,
            status: self.status,
            status_code: self.status_code,
            status_message: self.status_message,
            started_at: self.started_at,
            ended_at: self.ended_at,
            duration_ms: self.duration_ms,
            start_ms: self.start_ms,
            end_ms: self.end_ms,
            visual_start_ms: self.visual_start_ms,
            visual_end_ms: self.visual_end_ms,
            visual_duration_ms: self.visual_duration_ms,
            depth: self.depth,
            child_count: self.child_count,
            sort_index: self.sort_index,
            attributes: self.attributes,
            error_type: self.error_type,
            error_message: self.error_message,
        }
    }
}

fn parse_timestamp_ms(value: &str, utc_naive: bool) -> Option<f64> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(timestamp_millis_f64(parsed.timestamp_millis()));
    }
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").ok()?;
    if utc_naive {
        return Some(timestamp_millis_f64(
            Utc.from_utc_datetime(&naive).timestamp_millis(),
        ));
    }
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|parsed| timestamp_millis_f64(parsed.timestamp_millis()))
}

fn active_status_end_ms(status: &str, now_ms: f64) -> Option<f64> {
    matches!(status, "queued" | "planned" | "running").then_some(now_ms)
}

fn duration_from_bounds(start: Option<f64>, end: Option<f64>) -> f64 {
    start
        .zip(end)
        .map_or(0.0, |(start, end)| (end - start).max(0.0))
}

const fn min_optional_ms(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

const fn max_optional_ms(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn row_time_key(row: &WaterfallDraft) -> f64 {
    row.visual_start_ms.or(row.start_ms).unwrap_or(f64::MAX)
}

#[allow(clippy::cast_precision_loss)]
const fn timestamp_millis_f64(value: i64) -> f64 {
    value as f64
}
