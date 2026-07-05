impl Repository {
    pub(crate) fn persist_discovered_annotation_sync(
        &self,
        draft: &AnnotationIdentityDraft,
    ) -> Result<String> {
        let draft = draft.clone();
        self.with_sync_write(move |conn| {
            let annotation_id = Self::resolve_annotation_id(&conn, &draft)?;
            Self::upsert_annotation_identity(&conn, &annotation_id, &draft)?;
            Self::upsert_discovered_region(&conn, &annotation_id, &draft)?;
            Ok(annotation_id)
        })
    }

    fn resolve_annotation_id(conn: &Connection, draft: &AnnotationIdentityDraft) -> Result<String> {
        let existing: Option<String> = conn
            .query_row(
                "SELECT annotation_id
                 FROM document_annotation_identities
                 WHERE run_id = ? AND file_hash = ? AND page_no = ? AND source_region_key = ?
                 LIMIT 1",
                params![
                    draft.run_id,
                    draft.file_hash,
                    i64::from(draft.page_no),
                    draft.source_region_key
                ],
                |row| row.get(0),
            )
            .optional()?;
        Ok(existing.unwrap_or_else(new_persistence_id))
    }

    fn upsert_annotation_identity(
        conn: &Connection,
        annotation_id: &str,
        draft: &AnnotationIdentityDraft,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO document_annotation_identities(
                annotation_id, run_id, file_hash, page_no, engine_id, profile_id,
                source_region_key, discovery_index, label, x1, y1, x2, y2, created_at
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, now())
             ON CONFLICT(run_id, file_hash, page_no, source_region_key) DO UPDATE SET
                label = excluded.label, x1 = excluded.x1, y1 = excluded.y1,
                x2 = excluded.x2, y2 = excluded.y2, updated_at = now()",
            params![
                annotation_id,
                draft.run_id,
                draft.file_hash,
                i64::from(draft.page_no),
                draft.engine_id,
                draft.profile_id,
                draft.source_region_key,
                i64::from(draft.discovery_index),
                draft.label,
                draft.x1,
                draft.y1,
                draft.x2,
                draft.y2
            ],
        )?;
        Ok(())
    }

    fn upsert_discovered_region(
        conn: &Connection,
        annotation_id: &str,
        draft: &AnnotationIdentityDraft,
    ) -> Result<()> {
        conn.execute(
            "INSERT INTO document_regions(
                region_id, annotation_id, source_region_key, file_hash, page_no, engine_id,
                profile_id, label, x1, y1, x2, y2, source_span_start, source_span_end,
                content_markdown, content_html
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(region_id) DO UPDATE SET
                label = excluded.label, x1 = excluded.x1, y1 = excluded.y1,
                x2 = excluded.x2, y2 = excluded.y2,
                source_span_start = excluded.source_span_start,
                source_span_end = excluded.source_span_end,
                content_markdown = excluded.content_markdown,
                content_html = excluded.content_html",
            params![
                annotation_id,
                annotation_id,
                draft.source_region_key,
                draft.file_hash,
                i64::from(draft.page_no),
                draft.engine_id,
                draft.profile_id,
                draft.label,
                draft.x1,
                draft.y1,
                draft.x2,
                draft.y2,
                u64_to_i64_saturating(draft.span_start),
                u64_to_i64_saturating(draft.span_end),
                draft.content_markdown,
                draft.content_html
            ],
        )?;
        conn.execute(
            "INSERT INTO document_text_region_links(
                file_hash, page_no, region_id, annotation_id, text_start, text_end
             )
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(file_hash, page_no, region_id, text_start, text_end) DO UPDATE SET
                annotation_id = excluded.annotation_id",
            params![
                draft.file_hash,
                i64::from(draft.page_no),
                annotation_id,
                annotation_id,
                u64_to_i64_saturating(draft.span_start),
                u64_to_i64_saturating(draft.span_end)
            ],
        )?;
        Ok(())
    }
}
