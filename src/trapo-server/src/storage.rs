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
    workbench_types::{OverlayBox, TextRegionSpan},
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
include!("storage/diagnostics_types.rs");
include!("storage/diagnostics_writes.rs");
include!("storage/diagnostics_queries.rs");
include!("storage/diagnostics_rows.rs");
include!("storage/helpers.rs");
