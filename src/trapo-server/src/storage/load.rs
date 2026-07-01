impl Repository {
    fn load_runs(&self, conn: &Connection) -> Result<Vec<StoredRun>> {
        let mut statement = conn.prepare(
            "SELECT run_id, root_path, status, profile_id, engine_id, coalesce(model_id, ''),
              coalesce(runtime_id, ''), queued_files, processed_pages, total_pages, error
             FROM ingest_runs ORDER BY started_at DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(StoredRun {
                run_id: row.get(0)?,
                root_path: row.get(1)?,
                status: row.get(2)?,
                profile_id: row.get(3)?,
                engine_id: row.get(4)?,
                model_id: row.get(5)?,
                runtime_id: row.get(6)?,
                queued_files: i64_to_u32(row.get::<_, i64>(7)?),
                processed_pages: i64_to_u32(row.get::<_, i64>(8)?),
                total_pages: i64_to_u32(row.get::<_, i64>(9)?),
                error: row.get(10)?,
            })
        })?;
        collect_rows(rows)
    }

    fn load_documents(&self, conn: &Connection) -> Result<Vec<StoredDocument>> {
        let mut statement = conn.prepare(
            "SELECT f.file_hash, f.display_name, f.extension, f.size_bytes, f.page_count, f.status, f.error,
              coalesce(l.root_path, ''), coalesce(l.absolute_path, ''), coalesce(l.relative_path, '')
             FROM files f LEFT JOIN file_locations l ON l.file_hash = f.file_hash
             QUALIFY row_number() OVER (PARTITION BY f.file_hash ORDER BY l.observed_at DESC NULLS LAST) = 1
             ORDER BY f.display_name",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(StoredDocument {
                file_hash: row.get(0)?,
                display_name: row.get(1)?,
                extension: row.get(2)?,
                size_bytes: row.get::<_, i64>(3)? as u64,
                page_count: i64_to_u32(row.get::<_, i64>(4)?),
                status: row.get(5)?,
                error: row.get(6)?,
                root_path: row.get(7)?,
                absolute_path: row.get(8)?,
                relative_path: row.get(9)?,
            })
        })?;
        collect_rows(rows)
    }

    fn load_pages(&self, conn: &Connection) -> Result<Vec<StoredPage>> {
        let mut statement = conn.prepare(
            "SELECT p.file_hash, p.page_no, coalesce(p.width_px, 0), coalesce(p.height_px, 0),
              p.render_dpi, p.status, p.error, i.path, coalesce(o.cleaned_text, ''),
              coalesce(o.raw_text, '')
             FROM document_pages p
             LEFT JOIN document_preview_images i ON i.file_hash = p.file_hash AND i.page_no = p.page_no
              AND i.variant = 'source'
             LEFT JOIN document_page_ocr o ON o.file_hash = p.file_hash AND o.page_no = p.page_no
             ORDER BY p.file_hash, p.page_no",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(StoredPage {
                file_hash: row.get(0)?,
                page_no: i64_to_u32(row.get::<_, i64>(1)?),
                width_px: i64_to_u32(row.get::<_, i64>(2)?),
                height_px: i64_to_u32(row.get::<_, i64>(3)?),
                render_dpi: i64_to_u32(row.get::<_, i64>(4)?),
                status: row.get(5)?,
                error: row.get(6)?,
                preview_path: row.get(7)?,
                cleaned_text: row.get(8)?,
                raw_text: row.get(9)?,
                boxes: Vec::new(),
            })
        })?;
        collect_rows(rows)
    }
}
