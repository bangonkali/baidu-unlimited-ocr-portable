impl Repository {
    pub fn persist_realtime_event(
        &self,
        sequence: u64,
        event_type: &str,
        occurred_at: &str,
        payload: &Value,
    ) -> Result<()> {
        if !event_type.starts_with("ocr.page.") {
            return Ok(());
        }
        let conn = self.connect()?;
        let run_id = payload
            .get("run_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let file_hash = payload
            .get("file_hash")
            .and_then(Value::as_str)
            .map(str::to_string);
        let page_no = payload.get("page_no").and_then(Value::as_u64);
        conn.execute(
            "INSERT INTO ocr_stream_events(sequence, event_type, occurred_at, run_id, file_hash, page_no, payload_json)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(sequence) DO NOTHING",
            params![
                sequence as i64,
                event_type,
                occurred_at,
                run_id,
                file_hash,
                page_no.map(|value| value as i64),
                payload.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn list_ocr_stream_events(
        &self,
        run_id: Option<&str>,
        file_hash: Option<&str>,
        page_no: Option<u32>,
        since_sequence: Option<u64>,
        limit: u32,
    ) -> Result<Vec<StoredRealtimeEvent>> {
        let conn = self.connect()?;
        let page_no = page_no.map(i64::from);
        let since_sequence = since_sequence.map(|value| value as i64).unwrap_or(0);
        let limit = i64::from(limit.clamp(1, 100_000));
        let mut statement = conn.prepare(
            "SELECT sequence, event_type, occurred_at, run_id, file_hash, page_no, payload_json
             FROM ocr_stream_events
             WHERE sequence > ?
               AND (? IS NULL OR run_id = ?)
               AND (? IS NULL OR file_hash = ?)
               AND (? IS NULL OR page_no = ?)
             ORDER BY sequence ASC
             LIMIT ?",
        )?;
        let rows = statement.query_map(
            params![
                since_sequence,
                run_id,
                run_id,
                file_hash,
                file_hash,
                page_no,
                page_no,
                limit
            ],
            realtime_event_from_row,
        )?;
        collect_rows(rows)
    }
}

fn realtime_event_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<StoredRealtimeEvent> {
    let payload_json: String = row.get(6)?;
    Ok(StoredRealtimeEvent {
        sequence: row.get::<_, i64>(0)? as u64,
        event_type: row.get(1)?,
        occurred_at: row.get(2)?,
        run_id: row.get(3)?,
        file_hash: row.get(4)?,
        page_no: row.get::<_, Option<i64>>(5)?.map(i64_to_u32),
        payload: serde_json::from_str(&payload_json).unwrap_or(Value::Null),
    })
}
