struct DiagnosticSpanScope {
    span_id: String,
    started_at: String,
    started: Instant,
}

impl DiagnosticSpanScope {
    fn start() -> Self {
        Self {
            span_id: uuid::Uuid::new_v4().to_string(),
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
        let _ = self
            .inner
            .repository
            .insert_diagnostic_span(&DiagnosticSpanInsert {
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
        let _ = self
            .inner
            .repository
            .insert_diagnostic_event(&DiagnosticEventInsert {
                event_id: uuid::Uuid::new_v4().to_string(),
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
            });
    }

    fn upsert_diagnostic_work_unit(&self, draft: DiagnosticWorkUnitDraft<'_>) -> String {
        let work_unit_id =
            diagnostic_work_unit_id(draft.run_id, draft.file_hash, draft.page_no, draft.phase);
        let _ = self.inner.repository.upsert_work_unit(&WorkUnitUpsert {
            work_unit_id: work_unit_id.clone(),
            run_id: draft.run_id.to_string(),
            work_key: work_unit_id.clone(),
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
        });
        work_unit_id
    }

    fn start_diagnostic_work_unit(&self, run_id: &str, work_key: &str) {
        let _ = self.inner.repository.start_work_unit(run_id, work_key);
    }

    fn finish_diagnostic_work_unit(
        &self,
        run_id: &str,
        work_key: &str,
        status: &str,
        error: Option<&str>,
        result: Value,
    ) {
        let _ = self
            .inner
            .repository
            .finish_work_unit(run_id, work_key, status, &result, error);
    }

    fn record_model_lease(
        &self,
        run_id: &str,
        model_id: &str,
        runtime_id: &str,
        profile_id: &str,
        scope: DiagnosticSpanScope,
        error: Option<&str>,
    ) {
        let _ = self.inner.repository.insert_model_lease(
            run_id,
            "model",
            "local",
            model_id,
            if error.is_some() { "fallback" } else { "ok" },
            &json!({
                "runtime_id": runtime_id,
                "profile_id": profile_id,
                "started_at": scope.started_at,
                "duration_ms": scope.started.elapsed().as_secs_f64() * 1000.0,
                "error": error
            }),
        );
    }
}

fn diagnostic_work_unit_id(
    run_id: &str,
    file_hash: &str,
    page_no: Option<u32>,
    phase: &str,
) -> String {
    match page_no {
        Some(page_no) => format!("{run_id}:{file_hash}:page-{page_no}:{phase}"),
        None => format!("{run_id}:{file_hash}:file:{phase}"),
    }
}
