impl Repository {
    fn load_runs(conn: &Connection) -> Result<Vec<StoredRun>> {
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

    fn load_run_completion_manifests(
        conn: &Connection,
    ) -> Result<Vec<StoredRunCompletionManifest>> {
        let mut statement = conn.prepare(
            "SELECT run_id, completed_at, status, root_path, profile_id, engine_id,
              model_id, runtime_id, queued_files, processed_pages, total_pages,
              file_count, page_count, summary_json
             FROM ingest_run_completion_manifests
             ORDER BY completed_at DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(StoredRunCompletionManifest {
                run_id: row.get(0)?,
                completed_at: row.get(1)?,
                status: row.get(2)?,
                root_path: row.get(3)?,
                profile_id: row.get(4)?,
                engine_id: row.get(5)?,
                model_id: row.get(6)?,
                runtime_id: row.get(7)?,
                queued_files: i64_to_u32(row.get::<_, i64>(8)?),
                processed_pages: i64_to_u32(row.get::<_, i64>(9)?),
                total_pages: i64_to_u32(row.get::<_, i64>(10)?),
                file_count: i64_to_u32(row.get::<_, i64>(11)?),
                page_count: i64_to_u32(row.get::<_, i64>(12)?),
                summary: json_value(row.get::<_, String>(13)?.as_str()),
            })
        })?;
        collect_rows(rows)
    }

    fn load_documents(conn: &Connection) -> Result<Vec<StoredDocument>> {
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
                size_bytes: i64_to_u64(row.get::<_, i64>(3)?),
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

    fn load_run_documents(conn: &Connection) -> Result<Vec<StoredRunDocument>> {
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

    fn load_pages(conn: &Connection) -> Result<Vec<StoredPage>> {
        let mut statement = conn.prepare(
            "SELECT p.file_hash, p.page_no, coalesce(p.width_px, 0), coalesce(p.height_px, 0),
              p.render_dpi, p.status, p.error, i.path, coalesce(ro.run_id, ''),
              coalesce(ro.cleaned_text, lo.cleaned_text, ''), coalesce(ro.raw_text, lo.raw_text, '')
             FROM document_pages p
             LEFT JOIN document_preview_images i ON i.file_hash = p.file_hash AND i.page_no = p.page_no
              AND i.variant = 'source'
             LEFT JOIN (
               SELECT run_id, file_hash, page_no, cleaned_text, raw_text
               FROM document_run_page_ocr
               QUALIFY row_number() OVER (
                 PARTITION BY file_hash, page_no ORDER BY updated_at DESC, created_at DESC
               ) = 1
             ) ro ON ro.file_hash = p.file_hash AND ro.page_no = p.page_no
             LEFT JOIN (
               SELECT file_hash, page_no, cleaned_text, raw_text
               FROM document_page_ocr
               QUALIFY row_number() OVER (
                 PARTITION BY file_hash, page_no ORDER BY created_at DESC
               ) = 1
             ) lo ON lo.file_hash = p.file_hash AND lo.page_no = p.page_no
             ORDER BY p.file_hash, p.page_no",
        )?;
        let rows = statement.query_map([], |row| {
            let run_id = row.get::<_, String>(8)?;
            Ok(StoredPage {
                run_id: if run_id.is_empty() { None } else { Some(run_id) },
                file_hash: row.get(0)?,
                page_no: i64_to_u32(row.get::<_, i64>(1)?),
                width_px: i64_to_u32(row.get::<_, i64>(2)?),
                height_px: i64_to_u32(row.get::<_, i64>(3)?),
                render_dpi: i64_to_u32(row.get::<_, i64>(4)?),
                status: row.get(5)?,
                error: row.get(6)?,
                preview_path: row.get(7)?,
                cleaned_text: row.get(9)?,
                raw_text: row.get(10)?,
                boxes: Vec::new(),
                spans: Vec::new(),
            })
        })?;
        let mut pages = collect_rows(rows)?;
        for page in &mut pages {
            page.boxes = Self::load_page_boxes(
                conn,
                &page.file_hash,
                page.page_no,
                page.run_id.as_deref(),
            )?;
            page.spans = Self::load_page_spans(
                conn,
                &page.file_hash,
                page.page_no,
                page.run_id.as_deref(),
            )?;
        }
        Ok(pages)
    }

    fn load_page_boxes(
        conn: &Connection,
        file_hash: &str,
        page_no: u32,
        run_id: Option<&str>,
    ) -> Result<Vec<OverlayBox>> {
        let mut statement = conn.prepare(
            "SELECT coalesce(r.annotation_id, r.region_id), r.region_id,
              coalesce(r.source_region_key, ''), r.label, coalesce(r.category, r.label),
              coalesce(a.content_markdown, r.content_markdown, ''),
              coalesce(a.content_html, r.content_html), r.page_no, r.x1, r.y1, r.x2, r.y2,
              coalesce(v.hidden, false)
             FROM document_regions r
             LEFT JOIN document_region_annotations a ON a.region_id = coalesce(r.annotation_id, r.region_id)
             LEFT JOIN annotation_visibility_overrides v ON v.file_hash = r.file_hash
              AND v.page_no = r.page_no AND v.region_id = r.region_id
             WHERE r.file_hash = ? AND r.page_no = ? AND (? = '' OR r.run_id = ?)
             ORDER BY r.region_id",
        )?;
        let run_id = run_id.unwrap_or("");
        let rows = statement.query_map(params![file_hash, i64::from(page_no), run_id, run_id], |row| {
            let x1 = row.get::<_, f64>(8)?;
            let y1 = row.get::<_, f64>(9)?;
            let x2 = row.get::<_, f64>(10)?;
            let y2 = row.get::<_, f64>(11)?;
            Ok(OverlayBox {
                annotation_id: row.get(0)?,
                region_id: row.get(1)?,
                source_region_key: row.get(2)?,
                label: row.get(3)?,
                category: row.get(4)?,
                content_markdown: row.get(5)?,
                content_html: row.get(6)?,
                page_no: i64_to_u32(row.get::<_, i64>(7)?),
                left_percent: normalized_to_percent(x1),
                top_percent: normalized_to_percent(y1),
                width_percent: normalized_to_percent(x2 - x1),
                height_percent: normalized_to_percent(y2 - y1),
                hidden: row.get(12)?,
            })
        })?;
        collect_rows(rows)
    }

    fn load_page_spans(
        conn: &Connection,
        file_hash: &str,
        page_no: u32,
        run_id: Option<&str>,
    ) -> Result<Vec<TextRegionSpan>> {
        let mut statement = conn.prepare(
            "SELECT coalesce(annotation_id, region_id), region_id, page_no, text_start, text_end
             FROM document_text_region_links
             WHERE file_hash = ? AND page_no = ? AND (? = '' OR run_id = ?)
             ORDER BY text_start, region_id",
        )?;
        let run_id = run_id.unwrap_or("");
        let rows = statement.query_map(params![file_hash, i64::from(page_no), run_id, run_id], |row| {
            Ok(TextRegionSpan {
                annotation_id: row.get(0)?,
                region_id: row.get(1)?,
                source_region_key: String::new(),
                page_no: i64_to_u32(row.get::<_, i64>(2)?),
                start: i64_to_u64(row.get::<_, i64>(3)?),
                end: i64_to_u64(row.get::<_, i64>(4)?),
            })
        })?;
        collect_rows(rows)
    }
}

fn normalized_to_percent(value: f64) -> f64 {
    (value / 999.0 * 100.0).clamp(0.0, 100.0)
}
