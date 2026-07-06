impl Repository {
    pub(crate) async fn search_document_hashes(&self, query: &str, limit: u32) -> Result<Vec<String>> {
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

    pub(crate) async fn load_snapshot(&self) -> Result<StoredSnapshot> {
        self.with_read(move |conn| {
            Ok(StoredSnapshot {
                runs: Self::load_runs(&conn)?,
                completion_manifests: Self::load_run_completion_manifests(&conn)?,
                run_documents: Self::load_run_documents(&conn)?,
                documents: Self::load_documents(&conn)?,
                pages: Self::load_pages(&conn)?,
            })
        })
        .await
    }

    pub(crate) async fn completed_run_pages(&self, run_id: &str) -> Result<Vec<CompletedRunPage>> {
        let run_id = run_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT DISTINCT file_hash, page_no
                 FROM document_run_page_ocr
                 WHERE run_id = ? AND status = 'completed'
                 ORDER BY file_hash, page_no",
            )?;
            let rows = statement.query_map(params![run_id.as_str()], |row| {
                Ok(CompletedRunPage {
                    file_hash: row.get(0)?,
                    page_no: i64_to_u32(row.get::<_, i64>(1)?),
                })
            })?;
            collect_rows(rows)
        })
        .await
    }

    pub(crate) async fn load_document_regions_for_run(
        &self,
        file_hash: &str,
        run_id: &str,
    ) -> Result<Vec<OverlayBox>> {
        let file_hash = file_hash.to_string();
        let run_id = run_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT DISTINCT page_no
                 FROM document_regions
                 WHERE run_id = ? AND file_hash = ?
                 ORDER BY page_no",
            )?;
            let rows = statement.query_map(params![run_id.as_str(), file_hash.as_str()], |row| {
                Ok(i64_to_u32(row.get::<_, i64>(0)?))
            })?;
            let page_numbers = collect_rows(rows)?;
            let mut boxes = Vec::new();
            for page_no in page_numbers {
                boxes.extend(Self::load_page_boxes(
                    &conn,
                    &file_hash,
                    page_no,
                    Some(run_id.as_str()),
                )?);
            }
            Ok(boxes)
        })
        .await
    }

    pub(crate) async fn load_document_text_for_run(
        &self,
        file_hash: &str,
        run_id: &str,
    ) -> Result<Vec<PageTextRecord>> {
        let file_hash = file_hash.to_string();
        let run_id = run_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT page_no, cleaned_text
                 FROM document_run_page_ocr
                 WHERE run_id = ? AND file_hash = ?
                 ORDER BY page_no",
            )?;
            let rows = statement.query_map(params![run_id.as_str(), file_hash.as_str()], |row| {
                Ok(PageTextRecord {
                    page_no: i64_to_u32(row.get::<_, i64>(0)?),
                    text: row.get(1)?,
                    spans: Vec::new(),
                })
            })?;
            let mut pages = collect_rows(rows)?;
            for page in &mut pages {
                page.spans = Self::load_page_spans(
                    &conn,
                    &file_hash,
                    page.page_no,
                    Some(run_id.as_str()),
                )?;
            }
            Ok(pages)
        })
        .await
    }

    pub(crate) async fn upsert_page_metrics(&self, metrics: &OcrPageMetrics) -> Result<()> {
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
                u64_to_i64_saturating(metrics.token_count),
                metrics.avg_tps,
                u64_to_i64_saturating(metrics.elapsed_ms)
            ],
        )?;
        Ok(())
        })
        .await
    }

    pub(crate) async fn list_page_metrics(
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
