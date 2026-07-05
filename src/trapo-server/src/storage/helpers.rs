fn metrics_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<OcrPageMetrics> {
    Ok(OcrPageMetrics {
        run_id: row.get(0)?,
        file_hash: row.get(1)?,
        page_no: i64_to_u32(row.get::<_, i64>(2)?),
        model_id: row.get(3)?,
        runtime_id: row.get(4)?,
        status: row.get(5)?,
        token_count: i64_to_u64(row.get::<_, i64>(6)?),
        avg_tps: row.get(7)?,
        elapsed_ms: i64_to_u64(row.get::<_, i64>(8)?),
    })
}

fn collect_rows<T>(
    rows: duckdb::MappedRows<'_, impl FnMut(&duckdb::Row<'_>) -> duckdb::Result<T>>,
) -> Result<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

fn i64_to_u32(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value.max(0)).unwrap_or(u64::MAX)
}

fn u64_to_i64_saturating(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "database millisecond durations are exposed as f64 analytics values"
)]
const fn i64_to_f64_lossy(value: i64) -> f64 {
    value as f64
}
