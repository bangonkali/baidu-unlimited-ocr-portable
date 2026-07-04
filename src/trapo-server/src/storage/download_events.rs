impl Repository {
    pub async fn insert_download_event(&self, event: &DownloadEventInsert) -> Result<()> {
        let event = event.clone();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO download_events(event_id, download_id, owner_kind, owner_id, file_id,
                  file_name, target_path, source_url, event_type, status, downloaded_bytes,
                  total_bytes, error, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(event_id) DO NOTHING",
                params![
                    event.event_id.as_str(),
                    event.download_id.as_str(),
                    event.owner_kind.as_str(),
                    event.owner_id.as_str(),
                    event.file_id.as_str(),
                    event.file_name.as_str(),
                    event.target_path.as_str(),
                    event.source_url.as_str(),
                    event.event_type.as_str(),
                    event.status.as_str(),
                    event.downloaded_bytes as i64,
                    event.total_bytes.map(|value| value as i64),
                    event.error.as_deref(),
                    event.created_at.as_str()
                ],
            )?;
            Ok(())
        })
        .await
    }

    #[cfg(test)]
    pub async fn download_event_count(&self, download_id: &str, event_type: &str) -> Result<u64> {
        let download_id = download_id.to_string();
        let event_type = event_type.to_string();
        self.with_read(move |conn| {
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM download_events WHERE download_id = ? AND event_type = ?",
                params![download_id.as_str(), event_type.as_str()],
                |row| row.get(0),
            )?;
            Ok(count as u64)
        })
        .await
    }
}
