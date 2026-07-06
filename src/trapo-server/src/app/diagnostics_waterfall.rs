impl AppState {
    pub(crate) async fn diagnostic_waterfall(
        &self,
        request: DiagnosticTraceRequest,
    ) -> Result<DiagnosticWaterfallPayload> {
        let limit = limit_u32(request.limit, 20_000);
        let filter = DiagnosticTraceFilter {
            run_id: request.run_id.as_deref(),
            file_hash: request.file_hash.as_deref(),
            page_no: request.page_no,
            status: request.status.as_deref(),
            q: request.q.as_deref(),
            limit,
        };
        let (spans, _) = self.inner.repository.diagnostic_trace(&filter).await?;
        let work_units = self
            .inner
            .repository
            .diagnostic_work_units(request.run_id.as_deref(), limit)
            .await?;
        let pipeline_tasks = self
            .inner
            .repository
            .pipeline_tasks_for_diagnostics(request.run_id.as_deref(), limit)
            .await?;
        Ok(build_diagnostic_waterfall(
            request.run_id,
            spans,
            work_units,
            pipeline_tasks,
        ))
    }
}
