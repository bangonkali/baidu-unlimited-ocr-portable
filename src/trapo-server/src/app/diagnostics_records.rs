fn limit_u32(value: usize, max: u32) -> u32 {
    u32::try_from(value).map_or(max, |limit| limit.min(max))
}

fn diagnostic_run_record(row: DiagnosticRunRow) -> DiagnosticRunRecord {
    DiagnosticRunRecord {
        run_id: row.run_id,
        root_path: row.root_path,
        status: row.status,
        started_at: row.started_at,
        finished_at: row.finished_at,
        duration_ms: row.duration_ms,
        span_count: row.span_count,
        error_count: row.error_count,
        file_count: row.file_count,
        page_count: row.page_count,
    }
}

fn diagnostic_span_record(row: DiagnosticSpanRow) -> DiagnosticSpanRecord {
    DiagnosticSpanRecord {
        span_id: row.span_id,
        trace_id: row.trace_id,
        parent_span_id: row.parent_span_id,
        run_id: row.run_id,
        file_hash: row.file_hash,
        page_no: row.page_no,
        name: row.name,
        pipeline_step: row.pipeline_step,
        category: row.category,
        annotation_engine: row.annotation_engine,
        status: row.status,
        started_at: row.started_at,
        ended_at: row.ended_at,
        duration_ms: row.duration_ms,
        attributes: row.attributes,
        error_type: row.error_type,
        error_message: row.error_message,
        error_stack: row.error_stack,
    }
}

fn diagnostic_event_record(row: DiagnosticEventRow) -> DiagnosticEventRecord {
    DiagnosticEventRecord {
        event_id: row.event_id,
        trace_id: row.trace_id,
        span_id: row.span_id,
        run_id: row.run_id,
        file_hash: row.file_hash,
        page_no: row.page_no,
        timestamp: row.timestamp,
        event_type: row.event_type,
        name: row.name,
        severity: row.severity,
        message: row.message,
        attributes: row.attributes,
    }
}

fn diagnostic_work_unit_record(row: DiagnosticWorkUnitRow) -> DiagnosticWorkUnitRecord {
    DiagnosticWorkUnitRecord {
        work_unit_id: row.work_unit_id,
        run_id: row.run_id,
        work_key: row.work_key,
        file_hash: row.file_hash,
        filename: row.filename,
        source_path: row.source_path,
        page_no: row.page_no,
        phase: row.phase,
        engine: row.engine,
        provider: row.provider,
        model: row.model,
        profile: row.profile,
        execution_key: row.execution_key,
        artifact_variant: row.artifact_variant,
        status: row.status,
        attempt_count: row.attempt_count,
        started_at: row.started_at,
        finished_at: row.finished_at,
        duration_ms: row.duration_ms,
        error: row.error,
        result: row.result,
        metadata: row.metadata,
    }
}

fn diagnostic_model_lease_record(row: DiagnosticModelLeaseRow) -> DiagnosticModelLeaseRecord {
    DiagnosticModelLeaseRecord {
        lease_id: row.lease_id,
        run_id: row.run_id,
        execution_key: row.execution_key,
        provider: row.provider,
        model: row.model,
        requested_context_tokens: row.requested_context_tokens,
        verified_context_tokens: row.verified_context_tokens,
        status: row.status,
        started_at: row.started_at,
        finished_at: row.finished_at,
        duration_ms: row.duration_ms,
        error: row.error,
        metadata: row.metadata,
    }
}

fn diagnostic_trace_summary(
    run_id: Option<String>,
    spans: &[DiagnosticSpanRow],
    events: &[DiagnosticEventRow],
) -> DiagnosticTraceSummary {
    DiagnosticTraceSummary {
        run_id,
        span_count: spans.len() as u32,
        event_count: events.len() as u32,
        error_count: diagnostic_error_count(spans, events),
        total_duration_ms: spans.iter().map(|span| span.duration_ms).sum(),
    }
}

fn diagnostic_error_count(spans: &[DiagnosticSpanRow], events: &[DiagnosticEventRow]) -> u32 {
    let span_errors = spans
        .iter()
        .filter(|span| span.status == "error" || span.status == "failed" || span.error_message.is_some())
        .count();
    let event_errors = events
        .iter()
        .filter(|event| event.severity == "error" || event.event_type == "error")
        .count();
    (span_errors + event_errors) as u32
}

fn diagnostic_progress_summary(work_units: &[DiagnosticWorkUnitRow]) -> DiagnosticProgressSummary {
    let mut summary = DiagnosticProgressSummary {
        total_work_units: work_units.len() as u32,
        queued: 0,
        running: 0,
        completed: 0,
        failed: 0,
        cancelled: 0,
    };
    for unit in work_units {
        match unit.status.as_str() {
            "queued" | "planned" => summary.queued += 1,
            "running" => summary.running += 1,
            "completed" => summary.completed += 1,
            "failed" | "error" => summary.failed += 1,
            "cancelled" => summary.cancelled += 1,
            _ => {}
        }
    }
    summary
}

fn diagnostic_breakdown(
    spans: &[DiagnosticSpanRow],
    key: impl Fn(&DiagnosticSpanRow) -> &str,
) -> Vec<DiagnosticBreakdownRecord> {
    let mut map = BTreeMap::<String, (u32, f64)>::new();
    for span in spans {
        let entry = map.entry(key(span).to_string()).or_default();
        entry.0 += 1;
        entry.1 += span.duration_ms;
    }
    map.into_iter()
        .map(|(key, (count, total_duration_ms))| DiagnosticBreakdownRecord {
            key,
            count,
            total_duration_ms,
        })
        .collect()
}

fn diagnostic_slow_spans(spans: &[DiagnosticSpanRow]) -> Vec<DiagnosticSlowSpanRecord> {
    let mut slow = spans
        .iter()
        .map(|span| DiagnosticSlowSpanRecord {
            span_id: span.span_id.clone(),
            name: span.name.clone(),
            pipeline_step: span.pipeline_step.clone(),
            duration_ms: span.duration_ms,
            status: span.status.clone(),
        })
        .collect::<Vec<_>>();
    slow.sort_by(|left, right| right.duration_ms.total_cmp(&left.duration_ms));
    slow.truncate(20);
    slow
}

fn diagnostic_recommendations(
    error_count: u32,
    slow_spans: &[DiagnosticSlowSpanRecord],
) -> Vec<DiagnosticRecommendationRecord> {
    let mut recommendations = Vec::new();
    if error_count > 0 {
        recommendations.push(DiagnosticRecommendationRecord {
            severity: "error".to_string(),
            title: "Investigate failed spans".to_string(),
            detail: "The trace contains failed spans or error events. Filter by status to inspect the exact file, page, and step.".to_string(),
        });
    }
    if let Some(span) = slow_spans.first().filter(|span| span.duration_ms > 5_000.0) {
        recommendations.push(DiagnosticRecommendationRecord {
            severity: "warning".to_string(),
            title: "Review slowest pipeline step".to_string(),
            detail: format!(
                "{} spent {:.0} ms in {}.",
                span.name, span.duration_ms, span.pipeline_step
            ),
        });
    }
    recommendations
}
