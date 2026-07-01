mod migrations;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use duckdb::{Connection, OptionalExt, params};
use serde_json::Value;

use crate::{error::Result, workbench_types::OverlayBox};

#[derive(Debug, Clone)]
pub struct Repository {
    database_path: Arc<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct StoredRun {
    pub run_id: String,
    pub root_path: String,
    pub status: String,
    pub profile_id: String,
    pub engine_id: String,
    pub model_id: String,
    pub runtime_id: String,
    pub queued_files: u32,
    pub processed_pages: u32,
    pub total_pages: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StoredDocument {
    pub file_hash: String,
    pub display_name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub page_count: u32,
    pub status: String,
    pub error: Option<String>,
    pub root_path: String,
    pub absolute_path: String,
    pub relative_path: String,
}

#[derive(Debug, Clone)]
pub struct StoredPage {
    pub file_hash: String,
    pub page_no: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub render_dpi: u32,
    pub status: String,
    pub error: Option<String>,
    pub preview_path: Option<String>,
    pub cleaned_text: String,
    pub raw_text: String,
    pub boxes: Vec<OverlayBox>,
}

#[derive(Debug, Clone)]
pub struct OcrPageMetrics {
    pub run_id: String,
    pub file_hash: String,
    pub page_no: u32,
    pub model_id: String,
    pub runtime_id: String,
    pub status: String,
    pub token_count: u64,
    pub avg_tps: f64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Default)]
pub struct StoredSnapshot {
    pub runs: Vec<StoredRun>,
    pub documents: Vec<StoredDocument>,
    pub pages: Vec<StoredPage>,
}

include!("storage/core.rs");
include!("storage/writes.rs");
include!("storage/queries.rs");
include!("storage/internal.rs");
include!("storage/load.rs");

include!("storage/helpers.rs");
