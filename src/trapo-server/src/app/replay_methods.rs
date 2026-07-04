impl AppState {
    pub async fn ocr_replay(
        &self,
        run_id: Option<String>,
        file_hash: Option<String>,
        page_no: Option<u32>,
        since_sequence: Option<u64>,
        limit: usize,
    ) -> Result<OcrReplayPayload> {
        let limit = limit.min(10_000);
        let mut events = self
            .inner
            .repository
            .list_ocr_stream_events(
                run_id.as_deref(),
                file_hash.as_deref(),
                page_no,
                since_sequence,
                limit_u32(limit, 10_000),
            )
            .await?;
        events.extend(self.inner.hub.recent_ocr_events(
            run_id.as_deref(),
            file_hash.as_deref(),
            page_no,
            since_sequence,
            limit,
        ));
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

fn realtime_event_record(row: StoredRealtimeEvent) -> RealtimeEventRecord {
    RealtimeEventRecord {
        sequence: row.sequence,
        event_type: row.event_type,
        occurred_at: row.occurred_at,
        run_id: row.run_id,
        file_hash: row.file_hash,
        page_no: row.page_no,
        payload: row.payload,
    }
}
