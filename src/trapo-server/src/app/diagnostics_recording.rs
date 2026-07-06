struct DiagnosticSpanScope {
    span_id: String,
    started_at: String,
    started: Instant,
}

impl DiagnosticSpanScope {
    fn start() -> Self {
        Self {
            span_id: new_persistence_id(),
            started_at: Utc::now().to_rfc3339(),
            started: Instant::now(),
        }
    }
}

struct SpanFinish<'a> {
    run_id: &'a str,
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
    file_hash: &'a str,
    page_no: Option<u32>,
    phase: &'a str,
    model: &'a str,
    profile: &'a str,
    metadata: Value,
}

impl AppState {
    fn record_span(&self, scope: DiagnosticSpanScope, finish: SpanFinish<'_>) {
        let now = Utc::now().to_rfc3339();
        let repository = self.inner.repository.clone();
        let span = DiagnosticSpanInsert {
            span_id: scope.span_id,
            trace_id: finish.run_id.to_string(),
            parent_span_id: None,
            run_id: Some(finish.run_id.to_string()),
            file_hash: finish.file_hash.map(ToString::to_string),
            page_no: finish.page_no,
            name: finish.name.to_string(),
            pipeline_step: finish.pipeline_step.to_string(),
            category: finish.category.to_string(),
            annotation_engine: finish.engine.map(ToString::to_string),
            status: finish.status.to_string(),
            started_at: scope.started_at,
            ended_at: now,
            duration_ms: scope.started.elapsed().as_secs_f64() * 1000.0,
            attributes: finish.attributes,
            error_type: finish.error.map(|_| "AppError".to_string()),
            error_message: finish.error.map(ToString::to_string),
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
        let repository = self.inner.repository.clone();
        let event = DiagnosticEventInsert {
            event_id: new_persistence_id(),
            trace_id: run_id.to_string(),
            span_id: None,
            run_id: Some(run_id.to_string()),
            file_hash: file_hash.map(ToString::to_string),
            page_no,
            timestamp: Utc::now().to_rfc3339(),
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

    fn upsert_diagnostic_work_unit(&self, draft: DiagnosticWorkUnitDraft<'_>) -> String {
        let work_key =
            diagnostic_work_key(draft.run_id, draft.file_hash, draft.page_no, draft.phase);
        let repository = self.inner.repository.clone();
        let unit = WorkUnitUpsert {
            work_unit_id: new_persistence_id(),
            run_id: draft.run_id.to_string(),
            work_key: work_key.clone(),
            file_hash: Some(draft.file_hash.to_string()),
            page_no: draft.page_no,
            phase: draft.phase.to_string(),
            engine: ENGINE_ID.to_string(),
            provider: "local".to_string(),
            model: draft.model.to_string(),
            profile: Some(draft.profile.to_string()),
            execution_key: format!("{}:{}", draft.run_id, draft.phase),
            artifact_variant: None,
            metadata: draft.metadata,
        };
        self.spawn_background(async move {
            if let Err(error) = repository.upsert_work_unit(&unit).await {
                tracing::warn!(%error, "failed to persist diagnostic work unit");
            }
        });
        work_key
    }

    fn start_diagnostic_work_unit(&self, run_id: &str, work_key: &str) {
        let repository = self.inner.repository.clone();
        let run_id = run_id.to_string();
        let work_key = work_key.to_string();
        self.spawn_background(async move {
            if let Err(error) = repository.start_work_unit(&run_id, &work_key).await {
                tracing::warn!(%error, "failed to mark diagnostic work unit started");
            }
        });
    }

    fn finish_diagnostic_work_unit(
        &self,
        run_id: &str,
        work_key: &str,
        status: &str,
        error: Option<&str>,
        result: Value,
    ) {
        let repository = self.inner.repository.clone();
        let run_id = run_id.to_string();
        let work_key = work_key.to_string();
        let status = status.to_string();
        let error = error.map(str::to_string);
        self.spawn_background(async move {
            if let Err(error) = repository
                .finish_work_unit(&run_id, &work_key, &status, &result, error.as_deref())
                .await
            {
                tracing::warn!(%error, "failed to mark diagnostic work unit finished");
            }
        });
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
    file_hash: &str,
    page_no: Option<u32>,
    phase: &str,
) -> String {
    page_no.map_or_else(
        || format!("{run_id}:{file_hash}:file:{phase}"),
        |page_no| format!("{run_id}:{file_hash}:page-{page_no}:{phase}"),
    )
}
