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

    fn load_run_documents(&self, conn: &Connection) -> Result<Vec<StoredRunDocument>> {
        let mut statement = conn.prepare(
            "SELECT run_id, file_hash, ordinal
             FROM ingest_run_documents
             ORDER BY run_id, ordinal, file_hash",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(StoredRunDocument {
                run_id: row.get(0)?,
                file_hash: row.get(1)?,
                ordinal: i64_to_u32(row.get::<_, i64>(2)?),
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
                spans: Vec::new(),
            })
        })?;
        let mut pages = collect_rows(rows)?;
        for page in &mut pages {
            page.boxes = self.load_page_boxes(conn, &page.file_hash, page.page_no)?;
            page.spans = self.load_page_spans(conn, &page.file_hash, page.page_no)?;
        }
        Ok(pages)
    }

    fn load_page_boxes(
        &self,
        conn: &Connection,
        file_hash: &str,
        page_no: u32,
    ) -> Result<Vec<OverlayBox>> {
        let mut statement = conn.prepare(
            "SELECT r.region_id, r.label,
              coalesce(a.content_markdown, r.content_markdown, ''),
              coalesce(a.content_html, r.content_html), r.page_no, r.x1, r.y1, r.x2, r.y2,
              coalesce(v.hidden, false)
             FROM document_regions r
             LEFT JOIN document_region_annotations a ON a.region_id = r.region_id
             LEFT JOIN annotation_visibility_overrides v ON v.file_hash = r.file_hash
              AND v.page_no = r.page_no AND v.region_id = r.region_id
             WHERE r.file_hash = ? AND r.page_no = ?
             ORDER BY r.region_id",
        )?;
        let rows = statement.query_map(params![file_hash, i64::from(page_no)], |row| {
            let x1 = row.get::<_, f64>(5)?;
            let y1 = row.get::<_, f64>(6)?;
            let x2 = row.get::<_, f64>(7)?;
            let y2 = row.get::<_, f64>(8)?;
            Ok(OverlayBox {
                region_id: row.get(0)?,
                label: row.get(1)?,
                content_markdown: row.get(2)?,
                content_html: row.get(3)?,
                page_no: i64_to_u32(row.get::<_, i64>(4)?),
                left_percent: normalized_to_percent(x1),
                top_percent: normalized_to_percent(y1),
                width_percent: normalized_to_percent(x2 - x1),
                height_percent: normalized_to_percent(y2 - y1),
                hidden: row.get(9)?,
            })
        })?;
        collect_rows(rows)
    }

    fn load_page_spans(
        &self,
        conn: &Connection,
        file_hash: &str,
        page_no: u32,
    ) -> Result<Vec<TextRegionSpan>> {
        let mut statement = conn.prepare(
            "SELECT region_id, page_no, text_start, text_end
             FROM document_text_region_links
             WHERE file_hash = ? AND page_no = ?
             ORDER BY text_start, region_id",
        )?;
        let rows = statement.query_map(params![file_hash, i64::from(page_no)], |row| {
            Ok(TextRegionSpan {
                region_id: row.get(0)?,
                page_no: i64_to_u32(row.get::<_, i64>(1)?),
                start: i64_to_u64(row.get::<_, i64>(2)?),
                end: i64_to_u64(row.get::<_, i64>(3)?),
            })
        })?;
        collect_rows(rows)
    }
}

fn normalized_to_percent(value: f64) -> f64 {
    (value / 999.0 * 100.0).clamp(0.0, 100.0)
}
