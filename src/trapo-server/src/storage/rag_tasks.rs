impl Repository {
    pub(crate) async fn create_pipeline_task(
        &self,
        task_kind: &str,
        origin_run_id: Option<&str>,
        params: &Value,
    ) -> Result<PipelineTaskRow> {
        let task_id = new_persistence_id();
        let task_kind = task_kind.to_string();
        let origin_run_id = origin_run_id.map(str::to_string);
        let params_json = params.to_string();
        let queued_at = Utc::now().to_rfc3339();
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            let active_count = transaction.query_row(
                "SELECT count(*) FROM pipeline_tasks WHERE status IN ('queued', 'running')",
                [],
                |row| row.get::<_, i64>(0),
            )?;
            if active_count > 0 {
                return Err(AppError::Conflict(
                    "another ingest, text index, or embedding task is already active".to_string(),
                ));
            }
            transaction.execute(
                "INSERT INTO pipeline_tasks(task_id, task_kind, origin_run_id, status, params_json, queued_at)
                 VALUES (?, ?, ?, 'queued', ?, ?)",
                params![
                    task_id.as_str(),
                    task_kind.as_str(),
                    origin_run_id.as_deref(),
                    params_json.as_str(),
                    queued_at.as_str()
                ],
            )?;
            let row = PipelineTaskRow {
                task_id,
                task_kind,
                origin_run_id,
                status: "queued".to_string(),
                params: json_value(params_json.as_str()),
                result: Value::Object(serde_json::Map::default()),
                queued_at,
                started_at: None,
                finished_at: None,
                runner_id: None,
                error: None,
            };
            transaction.commit()?;
            Ok(row)
        })
        .await
    }

    pub(crate) async fn start_pipeline_task(
        &self,
        task_id: &str,
        runner_id: &str,
    ) -> Result<PipelineTaskRow> {
        let task_id = task_id.to_string();
        let runner_id = runner_id.to_string();
        let started_at = Utc::now().to_rfc3339();
        self.with_write(move |conn| {
            let changed = conn.execute(
                "UPDATE pipeline_tasks
                 SET status = 'running', started_at = ?, runner_id = ?
                 WHERE task_id = ? AND status = 'queued'",
                params![started_at.as_str(), runner_id.as_str(), task_id.as_str()],
            )?;
            if changed == 0 {
                return Err(AppError::Conflict("task is not queued".to_string()));
            }
            Self::pipeline_task_by_id(&conn, task_id.as_str())
        })
        .await
    }

    pub(crate) async fn finish_pipeline_task(
        &self,
        task_id: &str,
        status: &str,
        result: &Value,
        error: Option<&str>,
    ) -> Result<PipelineTaskRow> {
        let task_id = task_id.to_string();
        let status = status.to_string();
        let result_json = result.to_string();
        let error = error.map(str::to_string);
        let finished_at = Utc::now().to_rfc3339();
        self.with_write(move |conn| {
            conn.execute(
                "UPDATE pipeline_tasks
                 SET status = ?, result_json = ?, error = ?, finished_at = ?
                 WHERE task_id = ?",
                params![
                    status.as_str(),
                    result_json.as_str(),
                    error.as_deref(),
                    finished_at.as_str(),
                    task_id.as_str()
                ],
            )?;
            Self::pipeline_task_by_id(&conn, task_id.as_str())
        })
        .await
    }

    pub(crate) async fn active_pipeline_task(&self) -> Result<Option<PipelineTaskRow>> {
        self.with_read(|conn| {
            let mut statement = conn.prepare(
                "SELECT task_id, task_kind, origin_run_id, status, params_json, result_json,
                  queued_at, started_at, finished_at, runner_id, error
                 FROM pipeline_tasks
                 WHERE status IN ('queued', 'running')
                 ORDER BY queued_at LIMIT 1",
            )?;
            let mut rows = statement.query_map([], pipeline_task_from_row)?;
            rows.next().transpose().map_err(AppError::from)
        })
        .await
    }

    pub(crate) async fn pipeline_tasks_for_diagnostics(
        &self,
        origin_run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<PipelineTaskRow>> {
        let origin_run_id = origin_run_id.map(str::to_string);
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT task_id, task_kind, origin_run_id, status, params_json, result_json,
                  queued_at, started_at, finished_at, runner_id, error
                 FROM pipeline_tasks
                 WHERE (? IS NULL OR origin_run_id = ?)
                 ORDER BY queued_at DESC
                 LIMIT ?",
            )?;
            let rows = statement.query_map(
                params![
                    origin_run_id.as_deref(),
                    origin_run_id.as_deref(),
                    i64::from(limit.clamp(1, 10_000))
                ],
                pipeline_task_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }

    fn pipeline_task_by_id(conn: &Connection, task_id: &str) -> Result<PipelineTaskRow> {
        conn.query_row(
            "SELECT task_id, task_kind, origin_run_id, status, params_json, result_json,
              queued_at, started_at, finished_at, runner_id, error
             FROM pipeline_tasks WHERE task_id = ?",
            params![task_id],
            pipeline_task_from_row,
        )
        .map_err(AppError::from)
    }
}
