impl Repository {
    pub(crate) async fn upsert_rag_embedding_run(
        &self,
        row: &RagEmbeddingRunRow,
    ) -> Result<()> {
        let row = row.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO rag_embedding_runs(
                   embedding_run_id, task_id, source_run_id, model_id, requested_dimension,
                   actual_dimension, status, segments_total, segments_embedded, started_at,
                   finished_at, error, params_json
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(embedding_run_id) DO UPDATE SET
                   status = excluded.status,
                   segments_total = excluded.segments_total,
                   segments_embedded = excluded.segments_embedded,
                   finished_at = excluded.finished_at,
                   error = excluded.error,
                   params_json = excluded.params_json",
                params![
                    row.embedding_run_id.as_str(),
                    row.task_id.as_deref(),
                    row.source_run_id.as_str(),
                    row.model_id.as_str(),
                    i64::from(row.requested_dimension),
                    i64::from(row.actual_dimension),
                    row.status.as_str(),
                    i64::from(row.segments_total),
                    i64::from(row.segments_embedded),
                    row.started_at.as_str(),
                    row.finished_at.as_deref(),
                    row.error.as_deref(),
                    row.params.to_string().as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn insert_rag_embedding_vectors(
        &self,
        dimension: u32,
        vectors: &[RagEmbeddingVectorRow],
    ) -> Result<u32> {
        let table = vector_table_for_dimension(dimension)?.to_string();
        let vectors = vectors.to_vec();
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            for vector in &vectors {
                if u32::try_from(vector.embedding.len()).unwrap_or(u32::MAX) != dimension {
                    return Err(AppError::BadRequest(format!(
                        "embedding vector dimension mismatch: expected {dimension}"
                    )));
                }
                let literal = vector_literal(&vector.embedding);
                let sql = format!(
                    "INSERT INTO {table}(
                       embedding_run_id, source_run_id, segment_id, model_id, file_hash, page_no,
                       embedding, metadata_json, created_at
                     )
                     VALUES (?, ?, ?, ?, ?, ?, CAST(? AS FLOAT[{dimension}]), ?, ?)
                     ON CONFLICT(embedding_run_id, segment_id) DO UPDATE SET
                       embedding = excluded.embedding,
                       metadata_json = excluded.metadata_json"
                );
                transaction.execute(
                    sql.as_str(),
                    params![
                        vector.embedding_run_id.as_str(),
                        vector.source_run_id.as_str(),
                        vector.segment_id.as_str(),
                        vector.model_id.as_str(),
                        vector.file_hash.as_str(),
                        i64::from(vector.page_no),
                        literal.as_str(),
                        vector.metadata.to_string().as_str(),
                        Utc::now().to_rfc3339().as_str()
                    ],
                )?;
            }
            transaction.commit()?;
            create_hnsw_index_if_possible(&conn, table.as_str(), dimension);
            Ok(u32::try_from(vectors.len()).unwrap_or(u32::MAX))
        })
        .await
    }
}
