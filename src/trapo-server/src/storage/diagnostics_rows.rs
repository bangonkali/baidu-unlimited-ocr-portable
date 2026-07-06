fn work_unit_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<DiagnosticWorkUnitRow> {
    Ok(DiagnosticWorkUnitRow {
        work_unit_id: row.get(0)?,
        run_id: row.get(1)?,
        work_key: row.get(2)?,
        file_hash: row.get(3)?,
        filename: row.get(4)?,
        source_path: row.get(5)?,
        page_no: row.get::<_, Option<i64>>(6)?.map(i64_to_u32),
        phase: row.get(7)?,
        engine: row.get(8)?,
        provider: row.get(9)?,
        model: row.get(10)?,
        profile: row.get(11)?,
        execution_key: row.get(12)?,
        artifact_variant: row.get(13)?,
        status: row.get(14)?,
        attempt_count: i64_to_u32(row.get::<_, i64>(15)?),
        queued_at: row.get(16)?,
        started_at: row.get(17)?,
        finished_at: row.get(18)?,
        duration_ms: row.get(19)?,
        error: row.get(20)?,
        result: json_value(row.get::<_, String>(21)?.as_str()),
        metadata: json_value(row.get::<_, String>(22)?.as_str()),
    })
}

fn lease_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<DiagnosticModelLeaseRow> {
    Ok(DiagnosticModelLeaseRow {
        lease_id: row.get(0)?,
        run_id: row.get(1)?,
        execution_key: row.get(2)?,
        provider: row.get(3)?,
        model: row.get(4)?,
        requested_context_tokens: row.get::<_, Option<i64>>(5)?.map(i64_to_u32),
        verified_context_tokens: row.get::<_, Option<i64>>(6)?.map(i64_to_u32),
        status: row.get(7)?,
        started_at: row.get(8)?,
        finished_at: row.get(9)?,
        duration_ms: row.get(10)?,
        error: row.get(11)?,
        metadata: json_value(row.get::<_, String>(12)?.as_str()),
    })
}
