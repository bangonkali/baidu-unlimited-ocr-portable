impl Repository {
    pub fn insert_download_event(&self, event: &DownloadEventInsert) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO download_events(event_id, download_id, owner_kind, owner_id, file_id,
              file_name, target_path, source_url, event_type, status, downloaded_bytes,
              total_bytes, error, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(event_id) DO NOTHING",
            params![
                event.event_id,
                event.download_id,
                event.owner_kind,
                event.owner_id,
                event.file_id,
                event.file_name,
                event.target_path,
                event.source_url,
                event.event_type,
                event.status,
                event.downloaded_bytes as i64,
                event.total_bytes.map(|value| value as i64),
                event.error,
                event.created_at
            ],
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub fn download_event_count(&self, download_id: &str, event_type: &str) -> Result<u64> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM download_events WHERE download_id = ? AND event_type = ?",
            params![download_id, event_type],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}
