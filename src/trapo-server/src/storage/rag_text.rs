impl Repository {
    pub(crate) async fn replace_rag_text_segments(
        &self,
        source_run_id: &str,
        segments: &[RagTextSegmentRow],
    ) -> Result<u32> {
        let source_run_id = source_run_id.to_string();
        let segments = segments.to_vec();
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            transaction.execute(
                "DELETE FROM rag_text_segments WHERE source_run_id = ?",
                params![source_run_id.as_str()],
            )?;
            for segment in &segments {
                if segment.source_run_id != source_run_id {
                    return Err(AppError::BadRequest(
                        "text segment source_run_id does not match replace scope".to_string(),
                    ));
                }
                transaction.execute(
                    "INSERT INTO rag_text_segments(
                       segment_id, source_run_id, file_hash, page_no, segment_index,
                       annotation_id, category, text, token_estimate, text_start, text_end, source_kind
                     )
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        segment.segment_id.as_str(),
                        source_run_id.as_str(),
                        segment.file_hash.as_str(),
                        i64::from(segment.page_no),
                        i64::from(segment.segment_index),
                        segment.annotation_id.as_deref(),
                        segment.category.as_str(),
                        segment.text.as_str(),
                        i64::from(segment.token_estimate),
                        u64_to_i64_saturating(segment.text_start),
                        u64_to_i64_saturating(segment.text_end),
                        segment.source_kind.as_str()
                    ],
                )?;
            }
            transaction.commit()?;
            Ok(u32::try_from(segments.len()).unwrap_or(u32::MAX))
        })
        .await
    }

    pub(crate) async fn load_rag_text_segments(
        &self,
        source_run_id: &str,
    ) -> Result<Vec<RagTextSegmentRow>> {
        let source_run_id = source_run_id.to_string();
        self.with_read(move |conn| {
            let mut statement = conn.prepare(
                "SELECT segment_id, source_run_id, file_hash, page_no, segment_index,
                  annotation_id, category, text, token_estimate, text_start, text_end, source_kind
                 FROM rag_text_segments
                 WHERE source_run_id = ?
                 ORDER BY file_hash, page_no, segment_index",
            )?;
            let rows = statement.query_map(
                params![source_run_id.as_str()],
                rag_text_segment_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }

    pub(crate) async fn upsert_rag_text_index_run(
        &self,
        row: &RagTextIndexRunRow,
    ) -> Result<()> {
        let row = row.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO rag_text_index_runs(
                   text_index_run_id, task_id, source_run_id, status, segments_indexed,
                   started_at, finished_at, error
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(text_index_run_id) DO UPDATE SET
                   status = excluded.status,
                   segments_indexed = excluded.segments_indexed,
                   finished_at = excluded.finished_at,
                   error = excluded.error",
                params![
                    row.text_index_run_id.as_str(),
                    row.task_id.as_deref(),
                    row.source_run_id.as_str(),
                    row.status.as_str(),
                    i64::from(row.segments_indexed),
                    row.started_at.as_str(),
                    row.finished_at.as_deref(),
                    row.error.as_deref()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn refresh_rag_fts_index(
        &self,
        text_index_run_id: &str,
        source_run_id: &str,
    ) -> Result<u32> {
        let text_index_run_id = text_index_run_id.to_string();
        let source_run_id = source_run_id.to_string();
        self.with_write(move |conn| {
            let segments_indexed = conn.query_row(
                "SELECT count(*) FROM rag_text_segments WHERE source_run_id = ?",
                params![source_run_id.as_str()],
                |row| row.get::<_, i64>(0),
            )?;
            let index_result = conn.execute_batch(
                "PRAGMA create_fts_index(
                  'rag_text_segments', 'segment_id', 'text', 'category', overwrite = 1
                );",
            );
            let (status, error) = match index_result {
                Ok(()) => ("completed", None),
                Err(error) => ("degraded", Some(error.to_string())),
            };
            conn.execute(
                "INSERT INTO rag_fts_index_snapshots(
                   snapshot_id, text_index_run_id, source_run_id, index_name, status,
                   segments_indexed, created_at, error
                 )
                 VALUES (?, ?, ?, 'fts_main_rag_text_segments', ?, ?, ?, ?)",
                params![
                    new_persistence_id().as_str(),
                    text_index_run_id.as_str(),
                    source_run_id.as_str(),
                    status,
                    segments_indexed,
                    Utc::now().to_rfc3339().as_str(),
                    error.as_deref()
                ],
            )?;
            Ok(i64_to_u32(segments_indexed))
        })
        .await
    }
}
