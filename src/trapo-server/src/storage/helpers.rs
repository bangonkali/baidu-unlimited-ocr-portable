fn metrics_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<OcrPageMetrics> {
    Ok(OcrPageMetrics {
        run_id: row.get(0)?,
        file_hash: row.get(1)?,
        page_no: i64_to_u32(row.get::<_, i64>(2)?),
        model_id: row.get(3)?,
        runtime_id: row.get(4)?,
        status: row.get(5)?,
        token_count: row.get::<_, i64>(6)? as u64,
        avg_tps: row.get(7)?,
        elapsed_ms: row.get::<_, i64>(8)? as u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_and_persists_settings() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let repo = Repository::open(temp.path().join("trapo.duckdb"))?;
        repo.put_setting("selected_model_id", &Value::String("model".to_string()))?;
        assert_eq!(
            repo.setting_value("selected_model_id")?,
            Some(Value::String("model".to_string()))
        );
        Ok(())
    }
}
