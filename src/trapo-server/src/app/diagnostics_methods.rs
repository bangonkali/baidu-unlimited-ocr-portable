impl AppState {
    pub(crate) async fn diagnostics_runs(&self, limit: usize) -> Result<DiagnosticRunsPayload> {
        Ok(DiagnosticRunsPayload {
            runs: self
                .inner
                .repository
                .diagnostic_runs(limit_u32(limit, 500))
                .await?
                .into_iter()
                .map(diagnostic_run_record)
                .collect(),
        })
    }

    pub(crate) async fn diagnostic_trace(
        &self,
        request: DiagnosticTraceRequest,
    ) -> Result<DiagnosticTracePayload> {
        let filter = DiagnosticTraceFilter {
            run_id: request.run_id.as_deref(),
            file_hash: request.file_hash.as_deref(),
            page_no: request.page_no,
            status: request.status.as_deref(),
            q: request.q.as_deref(),
            limit: limit_u32(request.limit, 10_000),
        };
        let (spans, events) = self.inner.repository.diagnostic_trace(&filter).await?;
        let summary = diagnostic_trace_summary(request.run_id, &spans, &events);
        Ok(DiagnosticTracePayload {
            summary,
            spans: spans.into_iter().map(diagnostic_span_record).collect(),
            events: events.into_iter().map(diagnostic_event_record).collect(),
        })
    }

    pub(crate) async fn diagnostic_progress(
        &self,
        run_id: Option<String>,
        limit: usize,
    ) -> Result<DiagnosticProgressPayload> {
        let work_units = self
            .inner
            .repository
            .diagnostic_work_units(run_id.as_deref(), limit_u32(limit, 10_000))
            .await?;
        let model_leases = self
            .inner
            .repository
            .diagnostic_model_leases(run_id.as_deref(), limit_u32(limit, 2_000))
            .await?;
        let summary = diagnostic_progress_summary(&work_units);
        Ok(DiagnosticProgressPayload {
            summary,
            work_units: work_units.into_iter().map(diagnostic_work_unit_record).collect(),
            model_leases: model_leases
                .into_iter()
                .map(diagnostic_model_lease_record)
                .collect(),
        })
    }

    pub(crate) async fn diagnostic_analytics(
        &self,
        run_id: Option<String>,
        limit: usize,
    ) -> Result<DiagnosticAnalyticsPayload> {
        let filter = DiagnosticTraceFilter {
            run_id: run_id.as_deref(),
            file_hash: None,
            page_no: None,
            status: None,
            q: None,
            limit: limit_u32(limit, 25_000),
        };
        let (spans, events) = self.inner.repository.diagnostic_trace(&filter).await?;
        let span_count = usize_to_u32_saturating(spans.len());
        let event_count = usize_to_u32_saturating(events.len());
        let error_count = diagnostic_error_count(&spans, &events);
        let total_duration_ms = spans.iter().map(|span| span.duration_ms).sum::<f64>();
        let average_span_ms = if span_count == 0 {
            0.0
        } else {
            total_duration_ms / f64::from(span_count)
        };
        let slow_spans = diagnostic_slow_spans(&spans);
        let recommendations = diagnostic_recommendations(error_count, &slow_spans);
        Ok(DiagnosticAnalyticsPayload {
            summary: DiagnosticAnalyticsSummary {
                span_count,
                event_count,
                error_count,
                total_duration_ms,
                average_span_ms,
            },
            by_pipeline_step: diagnostic_breakdown(&spans, |span| &span.pipeline_step),
            by_category: diagnostic_breakdown(&spans, |span| &span.category),
            slow_spans,
            recommendations,
        })
    }

    pub(crate) async fn diagnostic_models(
        &self,
        run_id: Option<String>,
        limit: usize,
    ) -> Result<DiagnosticModelsPayload> {
        Ok(DiagnosticModelsPayload {
            model_leases: self
                .inner
                .repository
                .diagnostic_model_leases(run_id.as_deref(), limit_u32(limit, 2_000))
                .await?
                .into_iter()
                .map(diagnostic_model_lease_record)
                .collect(),
        })
    }
}

pub(crate) struct DiagnosticTraceRequest {
    pub(crate) run_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) status: Option<String>,
    pub(crate) q: Option<String>,
    pub(crate) limit: usize,
}
