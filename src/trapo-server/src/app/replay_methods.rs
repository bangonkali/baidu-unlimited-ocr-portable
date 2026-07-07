pub(crate) struct OcrReplayRequest {
    pub(crate) run_id: Option<String>,
    pub(crate) run_engine_id: Option<String>,
    pub(crate) file_hash: Option<String>,
    pub(crate) page_no: Option<u32>,
    pub(crate) since_sequence: Option<u64>,
    pub(crate) limit: usize,
}

impl AppState {
    pub(crate) async fn ocr_replay(&self, request: OcrReplayRequest) -> Result<OcrReplayPayload> {
        let limit = request.limit.min(10_000);
        let mut events = self
            .inner
            .repository
            .list_ocr_stream_events(
                request.run_id.as_deref(),
                request.file_hash.as_deref(),
                request.page_no,
                request.since_sequence,
                limit_u32(limit, 10_000),
            )
            .await?;
        events.extend(self.inner.hub.recent_ocr_events(
            request.run_id.as_deref(),
            request.file_hash.as_deref(),
            request.page_no,
            request.since_sequence,
            limit,
        ));
        if let Some(run_engine_id) = request.run_engine_id.as_deref() {
            events.retain(|event| event_run_engine_id(event) == Some(run_engine_id));
        }
        events.sort_by_key(|event| event.sequence);
        events.dedup_by_key(|event| event.sequence);
        events.truncate(limit);
        let next_since_sequence = events.last().map(|event| event.sequence);
        Ok(OcrReplayPayload {
            events: events.into_iter().map(realtime_event_record).collect(),
            next_since_sequence,
        })
    }
}

fn event_run_engine_id(event: &StoredRealtimeEvent) -> Option<&str> {
    event.payload.get("run_engine_id").and_then(Value::as_str)
}

fn realtime_event_record(row: StoredRealtimeEvent) -> RealtimeEventRecord {
    RealtimeEventRecord {
        event_id: row.event_id,
        sequence: row.sequence,
        event_type: row.event_type,
        occurred_at: row.occurred_at,
        run_id: row.run_id,
        file_hash: row.file_hash,
        page_no: row.page_no,
        payload: row.payload,
    }
}
