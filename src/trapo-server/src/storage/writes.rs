impl Repository {
    pub fn upsert_run(&self, run: &StoredRun) -> Result<()> {
        let conn = self.connect()?;
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
                run.run_id,
                run.root_path,
                run.status,
                run.profile_id,
                run.engine_id,
                run.error,
                i64::from(run.queued_files),
                i64::from(run.processed_pages),
                i64::from(run.total_pages),
                run.model_id,
                run.runtime_id
            ],
        )?;
        Ok(())
    }

    pub fn replace_run_documents(&self, run_id: &str, file_hashes: &[String]) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM ingest_run_documents WHERE run_id = ?",
            params![run_id],
        )?;
        for (index, file_hash) in file_hashes.iter().enumerate() {
            conn.execute(
                "INSERT INTO ingest_run_documents(run_id, file_hash, ordinal)
                 VALUES (?, ?, ?)
                 ON CONFLICT(run_id, file_hash) DO UPDATE SET ordinal = excluded.ordinal",
                params![run_id, file_hash, i64::try_from(index).unwrap_or(i64::MAX)],
            )?;
        }
        Ok(())
    }

    pub fn upsert_document(&self, document: &StoredDocument) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO files(file_hash, display_name, extension, size_bytes, page_count, status, error, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, current_timestamp)
             ON CONFLICT(file_hash) DO UPDATE SET display_name = excluded.display_name,
              extension = excluded.extension, size_bytes = excluded.size_bytes, page_count = excluded.page_count,
              status = excluded.status, error = excluded.error, updated_at = now()",
            params![
                document.file_hash,
                document.display_name,
                document.extension,
                document.size_bytes as i64,
                i64::from(document.page_count),
                document.status,
                document.error
            ],
        )?;
        conn.execute(
            "INSERT INTO file_locations(file_hash, root_path, absolute_path, relative_path, observed_at)
             VALUES (?, ?, ?, ?, current_timestamp)
             ON CONFLICT(file_hash, absolute_path) DO UPDATE SET root_path = excluded.root_path,
              relative_path = excluded.relative_path, observed_at = now()",
            params![
                document.file_hash,
                document.root_path,
                document.absolute_path,
                document.relative_path
            ],
        )?;
        Ok(())
    }

    pub fn upsert_page(&self, page: &StoredPage) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO document_pages(file_hash, page_no, width_px, height_px, render_dpi, status, error)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(file_hash, page_no) DO UPDATE SET width_px = excluded.width_px,
              height_px = excluded.height_px, render_dpi = excluded.render_dpi, status = excluded.status,
              error = excluded.error",
            params![
                page.file_hash,
                i64::from(page.page_no),
                i64::from(page.width_px),
                i64::from(page.height_px),
                i64::from(page.render_dpi),
                page.status,
                page.error
            ],
        )?;
        if let Some(path) = &page.preview_path {
            conn.execute(
                "INSERT INTO document_preview_images(file_hash, page_no, variant, path, width_px, height_px)
                 VALUES (?, ?, 'source', ?, ?, ?)
                 ON CONFLICT(file_hash, page_no, variant) DO UPDATE SET path = excluded.path,
                  width_px = excluded.width_px, height_px = excluded.height_px",
                params![
                    page.file_hash,
                    i64::from(page.page_no),
                    path,
                    i64::from(page.width_px),
                    i64::from(page.height_px)
                ],
            )?;
        }
        Ok(())
    }

    pub fn replace_page_ocr(
        &self,
        page: &StoredPage,
        engine_id: &str,
        profile_id: &str,
        elapsed_ms: u64,
    ) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO ocr_documents(file_hash, engine_id, profile_id, runtime_metadata, status, updated_at)
             VALUES (?, ?, ?, '{}'::JSON, ?, current_timestamp)
             ON CONFLICT(file_hash) DO UPDATE SET status = excluded.status, updated_at = now()",
            params![page.file_hash, engine_id, profile_id, page.status],
        )?;
        conn.execute(
            "INSERT INTO document_page_ocr(file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text,
              status, attempts, error, elapsed_ms, options)
             VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?, '{}'::JSON)
             ON CONFLICT(file_hash, page_no, engine_id, profile_id) DO UPDATE SET raw_text = excluded.raw_text,
              cleaned_text = excluded.cleaned_text, status = excluded.status, error = excluded.error,
              elapsed_ms = excluded.elapsed_ms",
            params![
                page.file_hash,
                i64::from(page.page_no),
                engine_id,
                profile_id,
                page.raw_text,
                page.cleaned_text,
                page.status,
                page.error,
                elapsed_ms as i64
            ],
        )?;
        self.replace_regions(&conn, page, engine_id, profile_id)?;
        Ok(())
    }
}
