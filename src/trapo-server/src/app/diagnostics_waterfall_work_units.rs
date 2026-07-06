fn matched_work_units(
    spans: &[DiagnosticSpanRow],
    work_units: &[DiagnosticWorkUnitRow],
) -> HashMap<String, DiagnosticWorkUnitRow> {
    let by_identity = work_units
        .iter()
        .flat_map(|unit| {
            [
                (unit.work_unit_id.clone(), unit.clone()),
                (unit.work_key.clone(), unit.clone()),
            ]
        })
        .collect::<HashMap<_, _>>();
    let by_location = work_units
        .iter()
        .filter_map(|unit| work_unit_location_key(unit).map(|key| (key, unit.clone())))
        .collect::<HashMap<_, _>>();
    let mut matched = HashMap::new();
    for span in spans {
        if let Some(unit) = span
            .work_unit_id
            .as_ref()
            .and_then(|id| by_identity.get(id))
            .or_else(|| span_location_key(span).and_then(|key| by_location.get(&key)))
        {
            matched.insert(unit.work_unit_id.clone(), unit.clone());
        }
    }
    matched
}

fn matching_work_unit_for_span<'a>(
    span: &DiagnosticSpanRow,
    work_unit_matches: &'a HashMap<String, DiagnosticWorkUnitRow>,
) -> Option<&'a DiagnosticWorkUnitRow> {
    if let Some(unit) = span
        .work_unit_id
        .as_ref()
        .and_then(|id| work_unit_matches.get(id))
    {
        return Some(unit);
    }
    work_unit_matches
        .values()
        .find(|unit| span_matches_work_unit(span, unit))
}

fn span_matches_work_unit(span: &DiagnosticSpanRow, unit: &DiagnosticWorkUnitRow) -> bool {
    let span_run_id = span.run_id.as_deref().unwrap_or(&span.trace_id);
    span_run_id == unit.run_id
        && span.file_hash == unit.file_hash
        && span.page_no == unit.page_no
        && span.pipeline_step == unit.phase
}

fn span_location_key(span: &DiagnosticSpanRow) -> Option<String> {
    let file_hash = span.file_hash.as_ref()?;
    let page_no = span.page_no?;
    let run_id = span.run_id.as_deref().unwrap_or(&span.trace_id);
    Some(format!("{run_id}:{file_hash}:{page_no}:{}", span.pipeline_step))
}

fn work_unit_location_key(unit: &DiagnosticWorkUnitRow) -> Option<String> {
    let file_hash = unit.file_hash.as_ref()?;
    let page_no = unit.page_no?;
    Some(format!(
        "{}:{file_hash}:{page_no}:{}",
        unit.run_id, unit.phase
    ))
}

fn span_attributes_with_work_unit(span: &DiagnosticSpanRow, unit: &DiagnosticWorkUnitRow) -> Value {
    let mut attributes = span.attributes.clone();
    let work_unit = json!({
        "work_unit_id": unit.work_unit_id.as_str(),
        "work_key": unit.work_key.as_str(),
        "status": unit.status.as_str(),
        "attempt_count": unit.attempt_count,
        "queued_at": unit.queued_at.as_str(),
        "started_at": unit.started_at.as_deref(),
        "finished_at": unit.finished_at.as_deref(),
        "duration_ms": unit.duration_ms,
        "result": &unit.result,
        "metadata": &unit.metadata
    });
    match &mut attributes {
        Value::Object(map) => {
            map.insert("work_unit".to_string(), work_unit);
            attributes
        }
        _ => json!({"span": attributes, "work_unit": work_unit}),
    }
}
