#[derive(Clone, Debug)]
struct ActivityContext {
    trace_id: String,
    span_id: String,
}

struct DiagnosticSpanScope {
    span_id: String,
    trace_id: Option<String>,
    parent_span_id: Option<String>,
    started_at: String,
    started_at_ms: i64,
    started: Instant,
    activity_kind: String,
}

impl DiagnosticSpanScope {
    fn start() -> Self {
        Self::start_internal(None, None)
    }

    fn start_root(trace_id: &str) -> Self {
        Self::start_internal(Some(trace_id), None)
    }

    fn start_child(parent: &ActivityContext) -> Self {
        Self::start_internal(Some(parent.trace_id.as_str()), Some(parent.span_id.as_str()))
    }

    fn context(&self, fallback_trace_id: &str) -> ActivityContext {
        ActivityContext {
            trace_id: self
                .trace_id
                .clone()
                .unwrap_or_else(|| fallback_trace_id.to_string()),
            span_id: self.span_id.clone(),
        }
    }

    fn start_internal(trace_id: Option<&str>, parent_span_id: Option<&str>) -> Self {
        let now = Utc::now();
        Self {
            span_id: new_persistence_id(),
            trace_id: trace_id.map(ToString::to_string),
            parent_span_id: parent_span_id.map(ToString::to_string),
            started_at: now.to_rfc3339(),
            started_at_ms: now.timestamp_millis(),
            started: Instant::now(),
            activity_kind: "internal".to_string(),
        }
    }
}

struct SpanFinish<'a> {
    run_id: &'a str,
    task_id: Option<&'a str>,
    work_unit_id: Option<&'a str>,
    parent_span_id: Option<&'a str>,
    file_hash: Option<&'a str>,
    page_no: Option<u32>,
    name: &'a str,
    pipeline_step: &'a str,
    category: &'a str,
    engine: Option<&'a str>,
    status: &'a str,
    error: Option<&'a str>,
    attributes: Value,
}

struct DiagnosticWorkUnitDraft<'a> {
    run_id: &'a str,
    run_engine_id: Option<&'a str>,
    file_hash: &'a str,
    page_no: Option<u32>,
    phase: &'a str,
    engine: &'a str,
    model: &'a str,
    profile: Option<&'a str>,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct DiagnosticWorkUnitHandle {
    id: String,
    key: String,
}

impl AppState {
    fn record_span(&self, scope: DiagnosticSpanScope, finish: SpanFinish<'_>) {
        let now = Utc::now();
        let ended_at_ms = now.timestamp_millis();
        let error_message = finish.error.map(ToString::to_string);
        let status_code = activity_status_code(finish.status, error_message.as_deref());
        let repository = self.inner.repository.clone();
        let span = DiagnosticSpanInsert {
            span_id: scope.span_id,
            trace_id: scope.trace_id.unwrap_or_else(|| finish.run_id.to_string()),
            parent_span_id: finish
                .parent_span_id
                .map(ToString::to_string)
                .or(scope.parent_span_id),
            task_id: finish.task_id.map(ToString::to_string),
            work_unit_id: finish.work_unit_id.map(ToString::to_string),
            span_kind: finish.category.to_string(),
            activity_kind: scope.activity_kind,
            run_id: Some(finish.run_id.to_string()),
            file_hash: finish.file_hash.map(ToString::to_string),
            page_no: finish.page_no,
            name: finish.name.to_string(),
            pipeline_step: finish.pipeline_step.to_string(),
            category: finish.category.to_string(),
            annotation_engine: finish.engine.map(ToString::to_string),
            status: finish.status.to_string(),
            status_code,
            status_message: error_message.clone(),
            started_at: scope.started_at,
            ended_at: now.to_rfc3339(),
            started_at_ms: scope.started_at_ms,
            ended_at_ms,
            duration_ms: scope.started.elapsed().as_secs_f64() * 1000.0,
            attributes: finish.attributes,
            resource: diagnostic_resource(),
            links: json!([]),
            error_type: finish.error.map(|_| "AppError".to_string()),
            error_message,
            error_stack: None,
        };
        self.spawn_background(async move {
            if let Err(error) = repository.insert_diagnostic_span(&span).await {
                tracing::warn!(%error, "failed to persist diagnostic span");
            }
        });
    }

    fn record_diagnostic_event(
        &self,
        run_id: &str,
        file_hash: Option<&str>,
        page_no: Option<u32>,
        severity: &str,
        message: &str,
    ) {
        let now = Utc::now();
        let repository = self.inner.repository.clone();
        let event = DiagnosticEventInsert {
            event_id: new_persistence_id(),
            trace_id: run_id.to_string(),
            span_id: None,
            run_id: Some(run_id.to_string()),
            file_hash: file_hash.map(ToString::to_string),
            page_no,
            timestamp: now.to_rfc3339(),
            timestamp_ms: now.timestamp_millis(),
            event_type: "log".to_string(),
            name: severity.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            attributes: json!({}),
        };
        self.spawn_background(async move {
            if let Err(error) = repository.insert_diagnostic_event(&event).await {
                tracing::warn!(%error, "failed to persist diagnostic event");
            }
        });
    }

    async fn upsert_diagnostic_work_unit(
        &self,
        draft: DiagnosticWorkUnitDraft<'_>,
    ) -> DiagnosticWorkUnitHandle {
        let work_key = diagnostic_work_key(
            draft.run_id,
            draft.run_engine_id,
            draft.file_hash,
            draft.page_no,
            draft.phase,
        );
        let unit = WorkUnitUpsert {
            work_unit_id: new_persistence_id(),
            run_id: draft.run_id.to_string(),
            run_engine_id: draft.run_engine_id.map(ToString::to_string),
            work_key: work_key.clone(),
            file_hash: Some(draft.file_hash.to_string()),
            page_no: draft.page_no,
            phase: draft.phase.to_string(),
            engine: draft.engine.to_string(),
            provider: "local".to_string(),
            model: draft.model.to_string(),
            profile: draft.profile.map(ToString::to_string),
            execution_key: draft
                .run_engine_id
                .map_or_else(|| format!("{}:{}", draft.run_id, draft.phase), ToString::to_string),
            artifact_variant: None,
            metadata: draft.metadata,
        };
        let id = match self.inner.repository.upsert_work_unit(&unit).await {
            Ok(id) => id,
            Err(error) => {
                tracing::warn!(%error, "failed to persist diagnostic work unit");
                unit.work_unit_id
            }
        };
        DiagnosticWorkUnitHandle { id, key: work_key }
    }

    async fn start_diagnostic_work_unit(&self, run_id: &str, work: &DiagnosticWorkUnitHandle) {
        if let Err(error) = self.inner.repository.start_work_unit(run_id, &work.key).await {
            tracing::warn!(%error, "failed to mark diagnostic work unit started");
        }
    }

    async fn finish_diagnostic_work_unit(
        &self,
        run_id: &str,
        work: &DiagnosticWorkUnitHandle,
        status: &str,
        error: Option<&str>,
        result: Value,
    ) {
        if let Err(error) = self
            .inner
            .repository
            .finish_work_unit(run_id, &work.key, status, &result, error)
            .await
        {
            tracing::warn!(%error, "failed to mark diagnostic work unit finished");
        }
    }

    fn record_model_lease(&self, lease: ModelLeaseDiagnostic<'_>, scope: &DiagnosticSpanScope) {
        let repository = self.inner.repository.clone();
        let status = if lease.error.is_some() { "fallback" } else { "ok" }.to_string();
        let metadata = json!({
            "runtime_id": lease.runtime_id,
            "profile_id": lease.profile_id,
            "started_at": scope.started_at.as_str(),
            "duration_ms": scope.started.elapsed().as_secs_f64() * 1000.0,
            "error": lease.error
        });
        let record = DiagnosticModelLeaseInsert {
            run_id: lease.run_id.to_string(),
            execution_key: "model".to_string(),
            provider: "local".to_string(),
            model: lease.model_id.to_string(),
            status,
            metadata,
        };
        self.spawn_background(async move {
            if let Err(error) = repository.insert_model_lease(&record).await {
                tracing::warn!(%error, "failed to persist model lease");
            }
        });
    }
}

fn activity_status_code(status: &str, error: Option<&str>) -> String {
    if error.is_some() || matches!(status, "error" | "failed") {
        return "error".to_string();
    }
    if matches!(status, "ok" | "completed" | "completed_with_errors") {
        return "ok".to_string();
    }
    "unset".to_string()
}

fn diagnostic_resource() -> Value {
    json!({
        "service.name": "trapo-server",
        "telemetry.sdk.name": "trapo-activity",
        "telemetry.sdk.language": "rust"
    })
}

#[derive(Clone, Copy)]
struct ModelLeaseDiagnostic<'a> {
    run_id: &'a str,
    model_id: &'a str,
    runtime_id: &'a str,
    profile_id: &'a str,
    error: Option<&'a str>,
}

fn diagnostic_work_key(
    run_id: &str,
    run_engine_id: Option<&str>,
    file_hash: &str,
    page_no: Option<u32>,
    phase: &str,
) -> String {
    let engine_scope = run_engine_id.map_or("run", |value| value);
    page_no.map_or_else(
        || format!("{run_id}:{engine_scope}:{file_hash}:file:{phase}"),
        |page_no| format!("{run_id}:{engine_scope}:{file_hash}:page-{page_no}:{phase}"),
    )
}
