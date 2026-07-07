#[derive(Clone, Copy)]
struct RagTaskDiagnosticContext<'a> {
    task: &'a PipelineTaskRow,
    parent_span_id: &'a str,
    pipeline_step: &'a str,
}

struct RagSpanInput<'a> {
    task: &'a PipelineTaskRow,
    parent_span_id: Option<&'a str>,
    file_hash: Option<&'a str>,
    page_no: Option<u32>,
    name: &'a str,
    pipeline_step: &'a str,
    span_kind: &'a str,
    engine: &'a str,
}

#[derive(Clone, Copy)]
struct RagPageSpanInput<'a> {
    task: &'a PipelineTaskRow,
    parent_span_id: &'a str,
    file_hash: &'a str,
    page_no: u32,
    name: &'a str,
    pipeline_step: &'a str,
    span_kind: &'a str,
}

struct PageSegmentGroup<'a> {
    file_hash: String,
    page_no: u32,
    segments: Vec<&'a RagTextSegmentRow>,
}

impl AppState {
    async fn upsert_rag_diagnostic_span(
        &self,
        scope: &DiagnosticSpanScope,
        input: RagSpanInput<'_>,
        status: &str,
        error: Option<&str>,
        attributes: Value,
    ) -> Result<()> {
        let run_id = input.task.origin_run_id.as_deref().unwrap_or("unscoped");
        let ended_at = if status == "running" {
            scope.started_at.clone()
        } else {
            Utc::now().to_rfc3339()
        };
        let duration_ms = if status == "running" {
            0.0
        } else {
            scope.started.elapsed().as_secs_f64() * 1000.0
        };
        self.inner
            .repository
            .insert_diagnostic_span(&DiagnosticSpanInsert {
                span_id: scope.span_id.clone(),
                trace_id: run_id.to_string(),
                parent_span_id: input.parent_span_id.map(ToString::to_string),
                task_id: Some(input.task.task_id.clone()),
                work_unit_id: None,
                span_kind: input.span_kind.to_string(),
                activity_kind: "internal".to_string(),
                run_id: Some(run_id.to_string()),
                file_hash: input.file_hash.map(ToString::to_string),
                page_no: input.page_no,
                name: input.name.to_string(),
                pipeline_step: input.pipeline_step.to_string(),
                category: input.span_kind.to_string(),
                annotation_engine: Some(input.engine.to_string()),
                status: status.to_string(),
                status_code: activity_status_code(status, error),
                status_message: error.map(ToString::to_string),
                started_at: scope.started_at.clone(),
                ended_at,
                started_at_ms: scope.started_at_ms,
                ended_at_ms: if status == "running" {
                    scope.started_at_ms
                } else {
                    Utc::now().timestamp_millis()
                },
                duration_ms,
                attributes,
                resource: diagnostic_resource(),
                links: json!([]),
                error_type: error.map(|_| "RagTask".to_string()),
                error_message: error.map(ToString::to_string),
                error_stack: None,
            })
            .await
    }
}

impl<'a> RagSpanInput<'a> {
    const fn task(task: &'a PipelineTaskRow, pipeline_step: &'a str) -> Self {
        Self {
            task,
            parent_span_id: None,
            file_hash: None,
            page_no: None,
            name: pipeline_step,
            pipeline_step,
            span_kind: "task",
            engine: "trapo-rag",
        }
    }

    const fn child(
        task: &'a PipelineTaskRow,
        parent_span_id: &'a str,
        name: &'a str,
        pipeline_step: &'a str,
        span_kind: &'a str,
    ) -> Self {
        Self {
            task,
            parent_span_id: Some(parent_span_id),
            file_hash: None,
            page_no: None,
            name,
            pipeline_step,
            span_kind,
            engine: "duckdb",
        }
    }

    const fn page(input: RagPageSpanInput<'a>) -> Self {
        Self {
            task: input.task,
            parent_span_id: Some(input.parent_span_id),
            file_hash: Some(input.file_hash),
            page_no: Some(input.page_no),
            name: input.name,
            pipeline_step: input.pipeline_step,
            span_kind: input.span_kind,
            engine: "llama.cpp",
        }
    }
}

fn page_segment_groups(segments: &[RagTextSegmentRow]) -> Vec<PageSegmentGroup<'_>> {
    let mut groups = BTreeMap::<(String, u32), Vec<&RagTextSegmentRow>>::new();
    for segment in segments {
        groups
            .entry((segment.file_hash.clone(), segment.page_no))
            .or_default()
            .push(segment);
    }
    groups
        .into_iter()
        .map(|((file_hash, page_no), segments)| PageSegmentGroup {
            file_hash,
            page_no,
            segments,
        })
        .collect()
}
