mod migration_sql;
mod migration_sql_downloads;
mod migrations;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::Utc;
use duckdb::{Connection, OptionalExt, params};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::{
    error::{AppError, Result},
    ids::{is_uuid_v7, new_persistence_id},
    workbench_types::{OverlayBox, PageTextRecord, TextRegionSpan},
};

const DB_READ_CONCURRENCY: usize = 8;
const DB_WRITE_CONCURRENCY: usize = 1;

/// DuckDB-backed repository for workbench state and telemetry.
#[derive(Debug, Clone)]
pub struct Repository {
    database_path: Arc<PathBuf>,
    shared_connection: Arc<Mutex<Connection>>,
    read_slots: Arc<Semaphore>,
    write_slots: Arc<Semaphore>,
    extension_capabilities: Arc<DbExtensionCapabilities>,
}

include!("storage/records.rs");
include!("storage/core.rs");
include!("storage/writes.rs");
include!("storage/queries.rs");
include!("storage/id_migrations.rs");
include!("storage/annotation_identities.rs");
include!("storage/internal.rs");
include!("storage/load.rs");
include!("storage/replay.rs");
include!("storage/download_events.rs");
include!("storage/rag_rows.rs");
include!("storage/rag_tasks.rs");
include!("storage/rag_models.rs");
include!("storage/rag_text.rs");
include!("storage/rag_vectors.rs");
include!("storage/rag_search.rs");
include!("storage/diagnostics_types.rs");
include!("storage/diagnostics_writes.rs");
include!("storage/diagnostics_queries.rs");
include!("storage/diagnostics_rows.rs");
include!("storage/helpers.rs");

#[cfg(test)]
include!("storage/test_fixtures.rs");

#[cfg(test)]
include!("storage/tests/basic.rs");

#[cfg(test)]
include!("storage/coverage_tests.rs");

#[cfg(test)]
include!("storage/migration_tests.rs");

#[cfg(test)]
include!("storage/rag_tests.rs");
