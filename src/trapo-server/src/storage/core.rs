impl Repository {
    pub fn open(database_path: impl Into<PathBuf>) -> Result<Self> {
        let database_path = database_path.into();
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let repository = Self {
            database_path: Arc::new(database_path),
        };
        repository.migrate()?;
        Ok(repository)
    }

    pub fn path(&self) -> &Path {
        &self.database_path
    }

    pub fn setting_value(&self, key: &str) -> Result<Option<Value>> {
        let conn = self.connect()?;
        let value: Option<String> = conn
            .query_row(
                "SELECT value::VARCHAR FROM settings WHERE key = ?",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value.and_then(|raw| serde_json::from_str(&raw).ok()))
    }

    pub fn put_setting(&self, key: &str, value: &Value) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO settings(key, value, updated_at) VALUES (?, CAST(? AS JSON), current_timestamp)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = now()",
            params![key, value.to_string()],
        )?;
        Ok(())
    }
}
