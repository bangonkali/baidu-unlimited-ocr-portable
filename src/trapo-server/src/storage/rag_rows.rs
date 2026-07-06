use std::fmt::Write as _;

fn pipeline_task_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<PipelineTaskRow> {
    let params_json = row.get::<_, String>(4)?;
    let result_json = row.get::<_, String>(5)?;
    Ok(PipelineTaskRow {
        task_id: row.get(0)?,
        task_kind: row.get(1)?,
        origin_run_id: row.get(2)?,
        status: row.get(3)?,
        params: json_value(params_json.as_str()),
        result: json_value(result_json.as_str()),
        queued_at: row.get(6)?,
        started_at: row.get(7)?,
        finished_at: row.get(8)?,
        runner_id: row.get(9)?,
        error: row.get(10)?,
    })
}

fn rag_embedding_model_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<RagEmbeddingModelRow> {
    let llama_params_json = row.get::<_, String>(14)?;
    Ok(RagEmbeddingModelRow {
        model_id: row.get(0)?,
        display_name: row.get(1)?,
        provider: row.get(2)?,
        repo_id: row.get(3)?,
        filename: row.get(4)?,
        revision: row.get(5)?,
        routing_origin: row.get(6)?,
        model_family: row.get(7)?,
        dimension: i64_to_u32(row.get::<_, i64>(8)?),
        context_tokens: i64_to_u32(row.get::<_, i64>(9)?),
        pooling: row.get(10)?,
        normalize: row.get(11)?,
        query_prefix: row.get(12)?,
        document_prefix: row.get(13)?,
        llama_params: json_value(llama_params_json.as_str()),
        recommended_vram_gb: row.get(15)?,
        active: row.get(16)?,
    })
}

fn rag_text_segment_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<RagTextSegmentRow> {
    Ok(RagTextSegmentRow {
        segment_id: row.get(0)?,
        source_run_id: row.get(1)?,
        file_hash: row.get(2)?,
        page_no: i64_to_u32(row.get::<_, i64>(3)?),
        segment_index: i64_to_u32(row.get::<_, i64>(4)?),
        annotation_id: row.get(5)?,
        category: row.get(6)?,
        text: row.get(7)?,
        token_estimate: i64_to_u32(row.get::<_, i64>(8)?),
        text_start: i64_to_u64(row.get::<_, i64>(9)?),
        text_end: i64_to_u64(row.get::<_, i64>(10)?),
        source_kind: row.get(11)?,
    })
}

fn vector_table_for_dimension(dimension: u32) -> Result<&'static str> {
    match dimension {
        128 => Ok("rag_embedding_vectors_128"),
        256 => Ok("rag_embedding_vectors_256"),
        512 => Ok("rag_embedding_vectors_512"),
        768 => Ok("rag_embedding_vectors_768"),
        1024 => Ok("rag_embedding_vectors_1024"),
        2560 => Ok("rag_embedding_vectors_2560"),
        4096 => Ok("rag_embedding_vectors_4096"),
        _ => Err(AppError::BadRequest(format!(
            "unsupported embedding dimension {dimension}"
        ))),
    }
}

fn vector_literal(values: &[f32]) -> String {
    let mut literal = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            literal.push(',');
        }
        let _ = write!(literal, "{value:.8}");
    }
    literal.push(']');
    literal
}

fn create_hnsw_index_if_possible(conn: &Connection, table: &str, dimension: u32) {
    let sql = format!(
        "SET hnsw_enable_experimental_persistence = true;
         CREATE INDEX IF NOT EXISTS idx_{table}_cosine
         ON {table} USING HNSW (embedding) WITH (metric = 'cosine');"
    );
    if let Err(error) = conn.execute_batch(sql.as_str()) {
        tracing::debug!(%error, %table, dimension, "HNSW index creation skipped");
    }
}
