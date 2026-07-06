impl Repository {
    pub(crate) async fn persist_discovered_annotations(
        &self,
        drafts: Vec<AnnotationIdentityDraft>,
    ) -> Result<()> {
        if drafts.is_empty() {
            return Ok(());
        }
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            for draft in &drafts {
                let annotation_id = Self::resolve_annotation_id(&transaction, draft)?;
                Self::upsert_annotation_identity(&transaction, &annotation_id, draft)?;
                Self::upsert_discovered_region(&transaction, &annotation_id, draft)?;
            }
            transaction.commit()?;
            Ok(())
        })
        .await
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
        Ok(existing
            .or_else(|| draft.annotation_id.clone())
            .unwrap_or_else(new_persistence_id))
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
                run_id, region_id, annotation_id, source_region_key, file_hash, page_no, engine_id,
                profile_id, label, x1, y1, x2, y2, source_span_start, source_span_end,
                content_markdown, content_html
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(region_id) DO UPDATE SET
                label = excluded.label, x1 = excluded.x1, y1 = excluded.y1,
                x2 = excluded.x2, y2 = excluded.y2,
                run_id = excluded.run_id,
                source_span_start = excluded.source_span_start,
                source_span_end = excluded.source_span_end,
                content_markdown = excluded.content_markdown,
                content_html = excluded.content_html",
            params![
                draft.run_id,
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
                run_id, file_hash, page_no, region_id, annotation_id, text_start, text_end
             )
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(file_hash, page_no, region_id, text_start, text_end) DO UPDATE SET
                run_id = excluded.run_id,
                annotation_id = excluded.annotation_id",
            params![
                draft.run_id,
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
