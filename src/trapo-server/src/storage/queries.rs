impl Repository {
    pub async fn search_document_hashes(&self, query: &str, limit: u32) -> Result<Vec<String>> {
        let query = query.to_string();
        self.with_read(move |conn| {
        let pattern = format!("%{}%", query.to_lowercase());
        let mut statement = conn.prepare(
            "SELECT DISTINCT f.file_hash FROM files f
             LEFT JOIN document_page_ocr o ON o.file_hash = f.file_hash
             LEFT JOIN file_locations l ON l.file_hash = f.file_hash
             WHERE lower(f.display_name || ' ' || coalesce(l.relative_path, '') || ' ' || coalesce(o.cleaned_text, '')) LIKE ?
             ORDER BY f.display_name LIMIT ?",
        )?;
        let rows = statement.query_map(params![pattern, i64::from(limit)], |row| row.get(0))?;
        collect_rows(rows)
        })
        .await
    }

    pub async fn load_snapshot(&self) -> Result<StoredSnapshot> {
        let repository = self.clone();
        self.with_read(move |conn| {
            Ok(StoredSnapshot {
                runs: repository.load_runs(&conn)?,
                run_documents: repository.load_run_documents(&conn)?,
                documents: repository.load_documents(&conn)?,
                pages: repository.load_pages(&conn)?,
            })
        })
        .await
    }

    pub async fn upsert_page_metrics(&self, metrics: &OcrPageMetrics) -> Result<()> {
        let metrics = metrics.clone();
        self.with_write(move |conn| {
        conn.execute(
            "INSERT INTO ocr_page_metrics(run_id, file_hash, page_no, engine_id, profile_id, model_id,
              runtime_id, runtime_platform, accelerator, status, token_count, avg_tps, elapsed_ms, started_at)
             VALUES (?, ?, ?, 'unlimited-ocr-ffi', '', ?, ?, '', '', ?, ?, ?, ?, current_timestamp::VARCHAR)
             ON CONFLICT(run_id, file_hash, page_no) DO UPDATE SET status = excluded.status,
              token_count = excluded.token_count, avg_tps = excluded.avg_tps, elapsed_ms = excluded.elapsed_ms,
              updated_at = now()",
            params![
                metrics.run_id,
                metrics.file_hash,
                i64::from(metrics.page_no),
                metrics.model_id,
                metrics.runtime_id,
                metrics.status,
                metrics.token_count as i64,
                metrics.avg_tps,
                metrics.elapsed_ms as i64
            ],
        )?;
        Ok(())
        })
        .await
    }

    pub async fn list_page_metrics(
        &self,
        run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<OcrPageMetrics>> {
        let run_id = run_id.map(str::to_string);
        self.with_read(move |conn| {
        let sql = if run_id.is_some() {
            "SELECT run_id, file_hash, page_no, model_id, coalesce(runtime_id, ''), status,
              token_count, avg_tps, elapsed_ms FROM ocr_page_metrics
             WHERE run_id = ? ORDER BY updated_at DESC LIMIT ?"
        } else {
            "SELECT run_id, file_hash, page_no, model_id, coalesce(runtime_id, ''), status,
              token_count, avg_tps, elapsed_ms FROM ocr_page_metrics
             ORDER BY updated_at DESC LIMIT ?"
        };
        let mut statement = conn.prepare(sql)?;
        if let Some(run_id) = run_id.as_deref() {
            let rows = statement.query_map(params![run_id, i64::from(limit)], metrics_from_row)?;
            collect_rows(rows)
        } else {
            let rows = statement.query_map(params![i64::from(limit)], metrics_from_row)?;
            collect_rows(rows)
        }
        })
        .await
    }
}
