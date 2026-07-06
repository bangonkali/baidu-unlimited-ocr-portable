impl Repository {
    pub(crate) async fn rag_fts_search(
        &self,
        query: &str,
        source_run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RagSearchHitRow>> {
        let query = query.to_string();
        let source_run_id = source_run_id.map(str::to_string);
        self.with_read(move |conn| {
            fts_search_with_index(&conn, query.as_str(), source_run_id.as_deref(), limit).or_else(
                |_| fts_search_with_like(&conn, query.as_str(), source_run_id.as_deref(), limit),
            )
        })
        .await
    }

    pub(crate) async fn rag_vss_search(
        &self,
        query_vector: &[f32],
        dimension: u32,
        model_id: &str,
        source_run_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<RagSearchHitRow>> {
        let table = vector_table_for_dimension(dimension)?.to_string();
        let literal = vector_literal(query_vector);
        let model_id = model_id.to_string();
        let source_run_id = source_run_id.map(str::to_string);
        self.with_read(move |conn| {
            let source_filter = source_run_id.as_deref().unwrap_or("");
            let sql = format!(
                "SELECT segment_id, file_hash, page_no, annotation_id, category, text, distance
                 FROM (
                   SELECT s.segment_id, s.file_hash, s.page_no, s.annotation_id, s.category, s.text,
                     array_cosine_distance(v.embedding, CAST(? AS FLOAT[{dimension}])) AS distance
                   FROM {table} v
                   JOIN rag_text_segments s ON s.segment_id = v.segment_id
                   WHERE v.model_id = ? AND (? = '' OR v.source_run_id = ?)
                 ) ranked
                 ORDER BY distance ASC
                 LIMIT ?"
            );
            let mut statement = conn.prepare(sql.as_str())?;
            let rows = statement.query_map(
                params![
                    literal.as_str(),
                    model_id.as_str(),
                    source_filter,
                    source_filter,
                    i64::from(limit)
                ],
                |row| {
                    let distance = row.get::<_, f64>(6)?;
                    Ok(RagSearchHitRow {
                        segment_id: row.get(0)?,
                        file_hash: row.get(1)?,
                        page_no: i64_to_u32(row.get::<_, i64>(2)?),
                        annotation_id: row.get(3)?,
                        category: row.get(4)?,
                        text: row.get(5)?,
                        score: 1.0 - distance,
                        hit_source: "vss".to_string(),
                        model_id: Some(model_id.clone()),
                    })
                },
            )?;
            collect_rows(rows)
        })
        .await
    }
}

fn fts_search_with_index(
    conn: &Connection,
    query: &str,
    source_run_id: Option<&str>,
    limit: u32,
) -> Result<Vec<RagSearchHitRow>> {
    let source_filter = source_run_id.unwrap_or("");
    let mut statement = conn.prepare(
        "SELECT segment_id, file_hash, page_no, annotation_id, category, text, score
         FROM (
           SELECT *, fts_main_rag_text_segments.match_bm25(segment_id, ?) AS score
           FROM rag_text_segments
           WHERE (? = '' OR source_run_id = ?)
         ) sq
         WHERE score IS NOT NULL
         ORDER BY score DESC
         LIMIT ?",
    )?;
    let rows = statement.query_map(
        params![query, source_filter, source_filter, i64::from(limit)],
        |row| {
            Ok(RagSearchHitRow {
                segment_id: row.get(0)?,
                file_hash: row.get(1)?,
                page_no: i64_to_u32(row.get::<_, i64>(2)?),
                annotation_id: row.get(3)?,
                category: row.get(4)?,
                text: row.get(5)?,
                score: row.get(6)?,
                hit_source: "fts".to_string(),
                model_id: None,
            })
        },
    )?;
    collect_rows(rows)
}

fn fts_search_with_like(
    conn: &Connection,
    query: &str,
    source_run_id: Option<&str>,
    limit: u32,
) -> Result<Vec<RagSearchHitRow>> {
    let source_filter = source_run_id.unwrap_or("");
    let pattern = format!("%{}%", query.to_lowercase());
    let mut statement = conn.prepare(
        "SELECT segment_id, file_hash, page_no, annotation_id, category, text
         FROM rag_text_segments
         WHERE lower(text || ' ' || category) LIKE ? AND (? = '' OR source_run_id = ?)
         ORDER BY file_hash, page_no, segment_index
         LIMIT ?",
    )?;
    let rows = statement.query_map(
        params![pattern.as_str(), source_filter, source_filter, i64::from(limit)],
        |row| {
            Ok(RagSearchHitRow {
                segment_id: row.get(0)?,
                file_hash: row.get(1)?,
                page_no: i64_to_u32(row.get::<_, i64>(2)?),
                annotation_id: row.get(3)?,
                category: row.get(4)?,
                text: row.get(5)?,
                score: 0.0,
                hit_source: "fts_fallback".to_string(),
                model_id: None,
            })
        },
    )?;
    collect_rows(rows)
}
