impl Repository {
    async fn with_read<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Connection) -> Result<T> + Send + 'static,
    {
        self.with_lane(self.read_slots.clone(), operation).await
    }

    async fn with_write<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Connection) -> Result<T> + Send + 'static,
    {
        self.with_lane(self.write_slots.clone(), operation).await
    }

    async fn with_lane<T, F>(&self, lane: Arc<Semaphore>, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Connection) -> Result<T> + Send + 'static,
    {
        let permit = lane
            .acquire_owned()
            .await
            .map_err(|error| AppError::Internal(format!("database lane closed: {error}")))?;
        let shared_connection = self.shared_connection.clone();
        tokio::task::spawn_blocking(move || {
            let conn = {
                let guard = shared_connection
                    .lock()
                    .map_err(|_| AppError::Internal("database connection mutex poisoned".to_string()))?;
                guard.try_clone()?
            };
            let _permit = permit;
            operation(conn)
        })
        .await
        .map_err(|error| AppError::Internal(format!("database worker failed: {error}")))?
    }

    async fn migrate(&self) -> Result<()> {
        self.with_write(|conn| {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
              id INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TIMESTAMP NOT NULL DEFAULT current_timestamp
            );",
        )?;
        for migration in migrations::MIGRATIONS {
            let applied: Option<i32> = conn
                .query_row(
                    "SELECT id FROM schema_migrations WHERE id = ?",
                    params![migration.id],
                    |row| row.get(0),
                )
                .optional()?;
            if applied.is_some() {
                continue;
            }
            conn.execute_batch(migration.sql)?;
            conn.execute(
                "INSERT INTO schema_migrations(id, name) VALUES (?, ?)",
                params![migration.id, migration.name],
            )?;
        }
        Self::migrate_generated_ids_to_uuid_v7(&conn)?;
        Ok(())
        })
        .await
    }

    fn replace_regions(
        conn: &Connection,
        run_id: &str,
        page: &StoredPage,
        engine_id: &str,
        profile_id: &str,
    ) -> Result<()> {
        Self::delete_page_regions(conn, run_id, page, engine_id, profile_id)?;
        Self::insert_page_regions(conn, run_id, page, engine_id, profile_id)?;
        Self::insert_text_region_links(conn, run_id, page)?;
        Ok(())
    }

    fn delete_page_regions(
        conn: &Connection,
        run_id: &str,
        page: &StoredPage,
        engine_id: &str,
        profile_id: &str,
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM document_text_region_links WHERE run_id = ? AND file_hash = ? AND page_no = ?",
            params![run_id, page.file_hash, i64::from(page.page_no)],
        )?;
        conn.execute(
            "DELETE FROM document_regions
             WHERE run_id = ? AND file_hash = ? AND page_no = ? AND engine_id = ? AND profile_id = ?",
            params![
                run_id,
                page.file_hash,
                i64::from(page.page_no),
                engine_id,
                profile_id
            ],
        )?;
        Ok(())
    }

    fn insert_page_regions(
        conn: &Connection,
        run_id: &str,
        page: &StoredPage,
        engine_id: &str,
        profile_id: &str,
    ) -> Result<()> {
        for box_record in &page.boxes {
            let source_span = page
                .spans
                .iter()
                .find(|span| span.annotation_id == box_record.annotation_id);
            let bbox_kind = box_record.storage_bbox_kind();
            let geometry_json = box_record.storage_geometry_json();
            let coordinate_space = box_record.storage_coordinate_space();
            let rotation_degrees = box_record.storage_rotation_degrees();
            conn.execute(
                "INSERT INTO document_regions(run_id, region_id, annotation_id, source_region_key,
                  file_hash, page_no, engine_id, profile_id, label, category,
                  bbox_kind, x1, y1, x2, y2, geometry_json, coordinate_space, rotation_degrees,
                  source_span_start, source_span_end, content_markdown, content_html)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    run_id,
                    box_record.region_id,
                    box_record.annotation_id,
                    box_record.source_region_key,
                    page.file_hash,
                    i64::from(page.page_no),
                    engine_id,
                    profile_id,
                    box_record.label,
                    box_record.category,
                    bbox_kind,
                    box_record.left_percent / 100.0 * 999.0,
                    box_record.top_percent / 100.0 * 999.0,
                    (box_record.left_percent + box_record.width_percent) / 100.0 * 999.0,
                    (box_record.top_percent + box_record.height_percent) / 100.0 * 999.0,
                    geometry_json,
                    coordinate_space,
                    rotation_degrees,
                    source_span.map(|span| u64_to_i64_saturating(span.start)),
                    source_span.map(|span| u64_to_i64_saturating(span.end)),
                    box_record.content_markdown,
                    box_record.content_html
                ],
            )?;
            conn.execute(
                "INSERT INTO document_region_annotations(region_id, file_hash, page_no, content_markdown, content_html)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(region_id) DO UPDATE SET content_markdown = excluded.content_markdown,
                  content_html = excluded.content_html, updated_at = now()",
                params![
                    box_record.annotation_id,
                    page.file_hash,
                    i64::from(page.page_no),
                    box_record.content_markdown,
                    box_record.content_html
                ],
            )?;
        }
        Ok(())
    }

    fn insert_text_region_links(conn: &Connection, run_id: &str, page: &StoredPage) -> Result<()> {
        for span in &page.spans {
            conn.execute(
                "INSERT INTO document_text_region_links(run_id, file_hash, page_no, region_id, annotation_id, text_start, text_end)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(file_hash, page_no, region_id, text_start, text_end) DO UPDATE SET
                    run_id = excluded.run_id,
                    annotation_id = excluded.annotation_id",
                params![
                    run_id,
                    page.file_hash,
                    i64::from(page.page_no),
                    span.region_id,
                    span.annotation_id,
                    u64_to_i64_saturating(span.start),
                    u64_to_i64_saturating(span.end)
                ],
            )?;
        }
        Ok(())
    }
}
