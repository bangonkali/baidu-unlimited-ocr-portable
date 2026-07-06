impl Repository {
    pub(crate) async fn upsert_run(&self, run: &StoredRun) -> Result<()> {
        let run = run.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ingest_runs(run_id, root_path, status, profile_id, engine_id, reprocess, error,
                  queued_files, processed_pages, total_pages, model_id, runtime_id)
                 VALUES (?, ?, ?, ?, ?, false, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(run_id) DO UPDATE SET status = excluded.status, error = excluded.error,
                  queued_files = excluded.queued_files, processed_pages = excluded.processed_pages,
                  total_pages = excluded.total_pages, model_id = excluded.model_id, runtime_id = excluded.runtime_id,
                  finished_at = CASE WHEN excluded.status IN ('completed','failed','cancelled','completed_with_errors')
                    THEN now() ELSE ingest_runs.finished_at END",
                params![
                    run.run_id.as_str(),
                    run.root_path.as_str(),
                    run.status.as_str(),
                    run.profile_id.as_str(),
                    run.engine_id.as_str(),
                    run.error.as_deref(),
                    i64::from(run.queued_files),
                    i64::from(run.processed_pages),
                    i64::from(run.total_pages),
                    run.model_id.as_str(),
                    run.runtime_id.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn replace_run_documents(&self, run_id: &str, file_hashes: &[String]) -> Result<()> {
        let run_id = run_id.to_string();
        let file_hashes = file_hashes.to_vec();
        self.with_write(move |conn| {
            conn.execute(
                "DELETE FROM ingest_run_documents WHERE run_id = ?",
                params![run_id.as_str()],
            )?;
            for (index, file_hash) in file_hashes.iter().enumerate() {
                conn.execute(
                    "INSERT INTO ingest_run_documents(run_id, file_hash, ordinal)
                     VALUES (?, ?, ?)
                     ON CONFLICT(run_id, file_hash) DO UPDATE SET ordinal = excluded.ordinal",
                    params![run_id.as_str(), file_hash.as_str(), i64::try_from(index).unwrap_or(i64::MAX)],
                )?;
            }
            Ok(())
        })
        .await
    }

    pub(crate) async fn upsert_document(&self, document: &StoredDocument) -> Result<()> {
        let document = document.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO files(file_hash, display_name, extension, size_bytes, page_count, status, error, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, current_timestamp)
                 ON CONFLICT(file_hash) DO UPDATE SET display_name = excluded.display_name,
                  extension = excluded.extension, size_bytes = excluded.size_bytes, page_count = excluded.page_count,
                  status = excluded.status, error = excluded.error, updated_at = now()",
                params![
                    document.file_hash.as_str(),
                    document.display_name.as_str(),
                    document.extension.as_str(),
                    u64_to_i64_saturating(document.size_bytes),
                    i64::from(document.page_count),
                    document.status.as_str(),
                    document.error.as_deref()
                ],
            )?;
            conn.execute(
                "INSERT INTO file_locations(file_hash, root_path, absolute_path, relative_path, observed_at)
                 VALUES (?, ?, ?, ?, current_timestamp)
                 ON CONFLICT(file_hash, absolute_path) DO UPDATE SET root_path = excluded.root_path,
                  relative_path = excluded.relative_path, observed_at = now()",
                params![
                    document.file_hash.as_str(),
                    document.root_path.as_str(),
                    document.absolute_path.as_str(),
                    document.relative_path.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn upsert_page(&self, page: &StoredPage) -> Result<()> {
        let page = page.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO document_pages(file_hash, page_no, width_px, height_px, render_dpi, status, error)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(file_hash, page_no) DO UPDATE SET width_px = excluded.width_px,
                  height_px = excluded.height_px, render_dpi = excluded.render_dpi, status = excluded.status,
                  error = excluded.error",
                params![
                    page.file_hash.as_str(),
                    i64::from(page.page_no),
                    i64::from(page.width_px),
                    i64::from(page.height_px),
                    i64::from(page.render_dpi),
                    page.status.as_str(),
                    page.error.as_deref()
                ],
            )?;
            if let Some(path) = &page.preview_path {
                conn.execute(
                    "INSERT INTO document_preview_images(file_hash, page_no, variant, path, width_px, height_px)
                     VALUES (?, ?, 'source', ?, ?, ?)
                     ON CONFLICT(file_hash, page_no, variant) DO UPDATE SET path = excluded.path,
                      width_px = excluded.width_px, height_px = excluded.height_px",
                    params![
                        page.file_hash.as_str(),
                        i64::from(page.page_no),
                        path.as_str(),
                        i64::from(page.width_px),
                        i64::from(page.height_px)
                    ],
                )?;
            }
            Ok(())
        })
        .await
    }

    pub(crate) async fn replace_page_ocr(
        &self,
        run_id: &str,
        page: &StoredPage,
        engine_id: &str,
        profile_id: &str,
        elapsed_ms: u64,
    ) -> Result<()> {
        let run_id = run_id.to_string();
        let page = page.clone();
        let engine_id = engine_id.to_string();
        let profile_id = profile_id.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO ocr_documents(file_hash, engine_id, profile_id, runtime_metadata, status, updated_at)
                 VALUES (?, ?, ?, '{}'::JSON, ?, current_timestamp)
                 ON CONFLICT(file_hash) DO UPDATE SET status = excluded.status, updated_at = now()",
                params![
                    page.file_hash.as_str(),
                    engine_id.as_str(),
                    profile_id.as_str(),
                    page.status.as_str()
                ],
            )?;
            conn.execute(
                "INSERT INTO document_page_ocr(file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text,
                  status, attempts, error, elapsed_ms, options)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?, '{}'::JSON)
                 ON CONFLICT(file_hash, page_no, engine_id, profile_id) DO UPDATE SET raw_text = excluded.raw_text,
                  cleaned_text = excluded.cleaned_text, status = excluded.status, error = excluded.error,
                  elapsed_ms = excluded.elapsed_ms",
                params![
                    page.file_hash.as_str(),
                    i64::from(page.page_no),
                    engine_id.as_str(),
                    profile_id.as_str(),
                    page.raw_text.as_str(),
                    page.cleaned_text.as_str(),
                    page.status.as_str(),
                    page.error.as_deref(),
                    u64_to_i64_saturating(elapsed_ms)
                ],
            )?;
            conn.execute(
                "INSERT INTO document_run_page_ocr(run_id, file_hash, page_no, engine_id, profile_id,
                  raw_text, cleaned_text, status, attempts, error, elapsed_ms, options, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, '{}'::JSON, current_timestamp)
                 ON CONFLICT(run_id, file_hash, page_no, engine_id, profile_id) DO UPDATE SET
                  raw_text = excluded.raw_text, cleaned_text = excluded.cleaned_text,
                  status = excluded.status, error = excluded.error, elapsed_ms = excluded.elapsed_ms,
                  updated_at = now()",
                params![
                    run_id.as_str(),
                    page.file_hash.as_str(),
                    i64::from(page.page_no),
                    engine_id.as_str(),
                    profile_id.as_str(),
                    page.raw_text.as_str(),
                    page.cleaned_text.as_str(),
                    page.status.as_str(),
                    page.error.as_deref(),
                    u64_to_i64_saturating(elapsed_ms)
                ],
            )?;
            Self::replace_regions(&conn, &run_id, &page, &engine_id, &profile_id)?;
            Ok(())
        })
        .await
    }
}
