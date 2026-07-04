impl Repository {
    #[cfg(test)]
    pub(crate) async fn persist_realtime_event(
        &self,
        sequence: u64,
        event_type: &str,
        occurred_at: &str,
        payload: &Value,
    ) -> Result<()> {
        if !event_type.starts_with("ocr.page.") {
            return Ok(());
        }
        let run_id = payload
            .get("run_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let file_hash = payload
            .get("file_hash")
            .and_then(Value::as_str)
            .map(str::to_string);
        let page_no = payload.get("page_no").and_then(Value::as_u64);
        self.persist_realtime_events(vec![StoredRealtimeEvent {
            sequence,
            event_type: event_type.to_string(),
            occurred_at: occurred_at.to_string(),
            run_id,
            file_hash,
            page_no: page_no.and_then(|value| u32::try_from(value).ok()),
            payload: payload.clone(),
        }])
        .await
    }

    pub(crate) async fn persist_realtime_events(&self, events: Vec<StoredRealtimeEvent>) -> Result<()> {
        let records: Vec<_> = events
            .into_iter()
            .filter(|event| event.event_type.starts_with("ocr.page."))
            .map(RealtimeEventWrite::from)
            .collect();
        if records.is_empty() {
            return Ok(());
        }
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            for record in &records {
                transaction.execute(
                    "INSERT INTO ocr_stream_events(sequence, event_type, occurred_at, run_id, file_hash, page_no, payload_json)
                     VALUES (?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(sequence) DO NOTHING",
                    params![
                        record.sequence,
                        record.event_type.as_str(),
                        record.occurred_at.as_str(),
                        record.run_id.as_deref(),
                        record.file_hash.as_deref(),
                        record.page_no,
                        record.payload_json.as_str()
                    ],
                )?;
            }
            transaction.commit()?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn list_ocr_stream_events(
        &self,
        run_id: Option<&str>,
        file_hash: Option<&str>,
        page_no: Option<u32>,
        since_sequence: Option<u64>,
        limit: u32,
    ) -> Result<Vec<StoredRealtimeEvent>> {
        let run_id = run_id.map(str::to_string);
        let file_hash = file_hash.map(str::to_string);
        self.with_read(move |conn| {
            let page_no = page_no.map(i64::from);
            let since_sequence = since_sequence.map_or(0, u64_to_i64_saturating);
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
                    run_id.as_deref(),
                    run_id.as_deref(),
                    file_hash.as_deref(),
                    file_hash.as_deref(),
                    page_no,
                    page_no,
                    limit
                ],
                realtime_event_from_row,
            )?;
            collect_rows(rows)
        })
        .await
    }
}

struct RealtimeEventWrite {
    sequence: i64,
    event_type: String,
    occurred_at: String,
    run_id: Option<String>,
    file_hash: Option<String>,
    page_no: Option<i64>,
    payload_json: String,
}

impl From<StoredRealtimeEvent> for RealtimeEventWrite {
    fn from(event: StoredRealtimeEvent) -> Self {
        Self {
            sequence: u64_to_i64_saturating(event.sequence),
            event_type: event.event_type,
            occurred_at: event.occurred_at,
            run_id: event.run_id,
            file_hash: event.file_hash,
            page_no: event.page_no.map(i64::from),
            payload_json: event.payload.to_string(),
        }
    }
}

fn realtime_event_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<StoredRealtimeEvent> {
    let payload_json: String = row.get(6)?;
    Ok(StoredRealtimeEvent {
        sequence: i64_to_u64(row.get::<_, i64>(0)?),
        event_type: row.get(1)?,
        occurred_at: row.get(2)?,
        run_id: row.get(3)?,
        file_hash: row.get(4)?,
        page_no: row.get::<_, Option<i64>>(5)?.map(i64_to_u32),
        payload: serde_json::from_str(&payload_json).unwrap_or(Value::Null),
    })
}
