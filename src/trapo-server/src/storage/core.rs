fn open_configured_database(database_path: &Path) -> duckdb::Result<Connection> {
    Connection::open(database_path) // skylos: ignore[SKY-D215] database_path is the configured app DuckDB file under the startup data directory.
}

impl Repository {
    pub async fn open(database_path: impl Into<PathBuf>) -> Result<Self> {
        let database_path = database_path.into();
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let connection_path = database_path.clone();
        let shared_connection = tokio::task::spawn_blocking(move || {
            open_configured_database(connection_path.as_path())
        })
        .await
        .map_err(|error| AppError::Internal(format!("database open worker failed: {error}")))??;
        let repository = Self {
            database_path: Arc::new(database_path),
            shared_connection: Arc::new(Mutex::new(shared_connection)),
            read_slots: Arc::new(Semaphore::new(DB_READ_CONCURRENCY)),
            write_slots: Arc::new(Semaphore::new(DB_WRITE_CONCURRENCY)),
        };
        repository.migrate().await?;
        Ok(repository)
    }

    pub fn path(&self) -> &Path {
        &self.database_path
    }

    pub async fn setting_value(&self, key: &str) -> Result<Option<Value>> {
        let key = key.to_string();
        self.with_read(move |conn| {
            let value: Option<String> = conn
                .query_row(
                    "SELECT value::VARCHAR FROM settings WHERE key = ?",
                    params![key],
                    |row| row.get(0),
                )
                .optional()?;
            Ok(value.and_then(|raw| serde_json::from_str(&raw).ok()))
        })
        .await
    }

    pub async fn put_setting(&self, key: &str, value: &Value) -> Result<()> {
        let key = key.to_string();
        let value = value.to_string();
        self.with_write(move |conn| {
            conn.execute(
                "INSERT INTO settings(key, value, updated_at) VALUES (?, CAST(? AS JSON), current_timestamp)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = now()",
                params![key, value],
            )?;
            Ok(())
        })
        .await
    }
}
