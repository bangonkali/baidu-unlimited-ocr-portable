impl AppState {
    async fn execute_text_index(
        &self,
        request: &TextIndexRequest,
        task: &PipelineTaskRow,
    ) -> Result<TextIndexResponse> {
        let task_scope = DiagnosticSpanScope::start();
        self.upsert_rag_diagnostic_span(
            &task_scope,
            RagSpanInput::task(task, "text_index"),
            "running",
            None,
            json!({}),
        )
        .await?;
        let started_at = Utc::now().to_rfc3339();
        let text_index_run_id = new_persistence_id();
        let segments = self
            .materialize_rag_text_segments_with_diagnostics(
                &request.source_run_id,
                Some(RagTaskDiagnosticContext {
                    task,
                    parent_span_id: &task_scope.span_id,
                    pipeline_step: "text_index",
                }),
            )
            .await?;
        let segments_indexed = self
            .inner
            .repository
            .replace_rag_text_segments(&request.source_run_id, &segments)
            .await?;
        self.upsert_text_index_run(TextIndexRunUpdate {
            request,
            task,
            text_index_run_id: &text_index_run_id,
            status: "running",
            segments_indexed,
            started_at: &started_at,
            finished_at: None,
        })
        .await?;
        let fts_segments = self
            .execute_text_index_fts_refresh(
                task,
                &task_scope,
                &text_index_run_id,
                &request.source_run_id,
                segments_indexed,
            )
            .await?;
        self.upsert_text_index_run(TextIndexRunUpdate {
            request,
            task,
            text_index_run_id: &text_index_run_id,
            status: "completed",
            segments_indexed: fts_segments,
            started_at: &started_at,
            finished_at: Some(Utc::now().to_rfc3339()),
        })
        .await?;
        self.finish_text_index_task(request, task, &task_scope, text_index_run_id, fts_segments)
            .await
    }

    async fn execute_text_index_fts_refresh(
        &self,
        task: &PipelineTaskRow,
        task_scope: &DiagnosticSpanScope,
        text_index_run_id: &str,
        source_run_id: &str,
        segments_indexed: u32,
    ) -> Result<u32> {
        let fts_scope = DiagnosticSpanScope::start();
        self.upsert_rag_diagnostic_span(
            &fts_scope,
            RagSpanInput::child(
                task,
                &task_scope.span_id,
                "Refresh DuckDB FTS",
                "text_index",
                "fts",
            ),
            "running",
            None,
            json!({"segments_total": segments_indexed}),
        )
        .await?;
        let fts_segments = self
            .inner
            .repository
            .refresh_rag_fts_index(text_index_run_id, source_run_id)
            .await?;
        self.upsert_rag_diagnostic_span(
            &fts_scope,
            RagSpanInput::child(
                task,
                &task_scope.span_id,
                "Refresh DuckDB FTS",
                "text_index",
                "fts",
            ),
            "ok",
            None,
            json!({"segments_indexed": fts_segments}),
        )
        .await?;
        Ok(fts_segments)
    }

    async fn upsert_text_index_run(&self, update: TextIndexRunUpdate<'_>) -> Result<()> {
        self.inner
            .repository
            .upsert_rag_text_index_run(&RagTextIndexRunRow {
                text_index_run_id: update.text_index_run_id.to_string(),
                task_id: Some(update.task.task_id.clone()),
                source_run_id: update.request.source_run_id.clone(),
                status: update.status.to_string(),
                segments_indexed: update.segments_indexed,
                started_at: update.started_at.to_string(),
                finished_at: update.finished_at,
                error: None,
            })
            .await
    }

    async fn finish_text_index_task(
        &self,
        request: &TextIndexRequest,
        task: &PipelineTaskRow,
        task_scope: &DiagnosticSpanScope,
        text_index_run_id: String,
        fts_segments: u32,
    ) -> Result<TextIndexResponse> {
        let finished_task = self
            .inner
            .repository
            .finish_pipeline_task(
                &task.task_id,
                "completed",
                &json!({"segments_indexed": fts_segments}),
                None,
            )
            .await?;
        self.upsert_rag_diagnostic_span(
            task_scope,
            RagSpanInput::task(task, "text_index"),
            "ok",
            None,
            json!({"segments_indexed": fts_segments, "text_index_run_id": text_index_run_id.clone()}),
        )
        .await?;
        Ok(TextIndexResponse {
            task: pipeline_task_record(finished_task),
            text_index_run_id,
            source_run_id: request.source_run_id.clone(),
            segments_indexed: fts_segments,
            status: "completed".to_string(),
        })
    }
}

struct TextIndexRunUpdate<'a> {
    request: &'a TextIndexRequest,
    task: &'a PipelineTaskRow,
    text_index_run_id: &'a str,
    status: &'a str,
    segments_indexed: u32,
    started_at: &'a str,
    finished_at: Option<String>,
}
