fn open_configured_database(database_path: &Path) -> duckdb::Result<Connection> {
    Connection::open(database_path) // skylos: ignore[SKY-D215] database_path is the configured app DuckDB file under the startup data directory.
}

impl Repository {
    /// Opens the repository and applies pending schema migrations.
    ///
    /// # Errors
    ///
    /// Returns an error when the `DuckDB` file cannot be opened or migrations fail.
    pub async fn open(database_path: impl Into<PathBuf>) -> Result<Self> {
        let database_path = database_path.into();
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let connection_path = database_path.clone();
        let (shared_connection, extension_capabilities) = tokio::task::spawn_blocking(move || {
            open_configured_database(connection_path.as_path())
                .map(|connection| {
                    let extension_capabilities = configure_duckdb_extensions(&connection);
                    (connection, extension_capabilities)
                })
        })
        .await
        .map_err(|error| AppError::Internal(format!("database open worker failed: {error}")))??;
        let repository = Self {
            database_path: Arc::new(database_path),
            shared_connection: Arc::new(Mutex::new(shared_connection)),
            read_slots: Arc::new(Semaphore::new(DB_READ_CONCURRENCY)),
            write_slots: Arc::new(Semaphore::new(DB_WRITE_CONCURRENCY)),
            extension_capabilities: Arc::new(extension_capabilities),
        };
        repository.migrate().await?;
        Ok(repository)
    }

    #[must_use]
    pub(crate) fn path(&self) -> &Path {
        &self.database_path
    }

    #[must_use]
    pub(crate) fn extension_capabilities(&self) -> DbExtensionCapabilities {
        (*self.extension_capabilities).clone()
    }

    pub(crate) async fn setting_value(&self, key: &str) -> Result<Option<Value>> {
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

    pub(crate) async fn put_setting(&self, key: &str, value: &Value) -> Result<()> {
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

    pub(crate) async fn checkpoint(&self) -> Result<()> {
        self.with_write(|conn| {
            conn.execute_batch("CHECKPOINT;")?;
            Ok(())
        })
        .await
    }
}

fn configure_duckdb_extensions(conn: &Connection) -> DbExtensionCapabilities {
    let (fts_loaded, fts_error) = load_duckdb_extension(conn, "fts");
    let (vss_loaded, vss_error) = load_duckdb_extension(conn, "vss");
    if vss_loaded {
        let _ = conn.execute_batch("SET hnsw_enable_experimental_persistence = true;");
    }
    DbExtensionCapabilities {
        fts_loaded,
        fts_error,
        vss_loaded,
        vss_error,
        duckpgq_loaded: false,
        duckpgq_error: Some("duckpgq initialization deferred until graph queries are introduced".to_string()),
    }
}

fn load_duckdb_extension(conn: &Connection, name: &str) -> (bool, Option<String>) {
    let install_sql = format!("INSTALL {name};");
    let load_sql = format!("LOAD {name};");
    match conn.execute_batch(&install_sql).and_then(|()| conn.execute_batch(&load_sql)) {
        Ok(()) => (true, None),
        Err(install_error) => match conn.execute_batch(&load_sql) {
            Ok(()) => (true, Some(format!("install skipped or failed before load succeeded: {install_error}"))),
            Err(load_error) => (false, Some(format!("{install_error}; load failed: {load_error}"))),
        },
    }
}
