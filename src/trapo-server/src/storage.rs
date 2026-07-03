mod migrations;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use duckdb::{Connection, OptionalExt, params};
use serde_json::Value;

use crate::{
    error::Result,
    workbench_types::{OverlayBox, TextRegionSpan},
};

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
    pub spans: Vec<TextRegionSpan>,
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

#[derive(Debug, Clone)]
pub struct StoredRealtimeEvent {
    pub sequence: u64,
    pub event_type: String,
    pub occurred_at: String,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct DiagnosticRunRow {
    pub run_id: String,
    pub root_path: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: f64,
    pub span_count: u32,
    pub error_count: u32,
    pub file_count: u32,
    pub page_count: u32,
}

#[derive(Debug, Clone)]
pub struct DiagnosticSpanRow {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub name: String,
    pub pipeline_step: String,
    pub category: String,
    pub annotation_engine: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: f64,
    pub attributes: Value,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_stack: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticEventRow {
    pub event_id: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub timestamp: String,
    pub event_type: String,
    pub name: String,
    pub severity: String,
    pub message: String,
    pub attributes: Value,
}

#[derive(Debug, Clone)]
pub struct DiagnosticWorkUnitRow {
    pub work_unit_id: String,
    pub run_id: String,
    pub work_key: String,
    pub file_hash: Option<String>,
    pub filename: Option<String>,
    pub source_path: Option<String>,
    pub page_no: Option<u32>,
    pub phase: String,
    pub engine: String,
    pub provider: String,
    pub model: String,
    pub profile: Option<String>,
    pub execution_key: String,
    pub artifact_variant: Option<String>,
    pub status: String,
    pub attempt_count: u32,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    pub result: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct DiagnosticModelLeaseRow {
    pub lease_id: String,
    pub run_id: String,
    pub execution_key: String,
    pub provider: String,
    pub model: String,
    pub requested_context_tokens: Option<u32>,
    pub verified_context_tokens: Option<u32>,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct WorkUnitUpsert {
    pub work_unit_id: String,
    pub run_id: String,
    pub work_key: String,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub phase: String,
    pub engine: String,
    pub provider: String,
    pub model: String,
    pub profile: Option<String>,
    pub execution_key: String,
    pub artifact_variant: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct DiagnosticSpanInsert {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub name: String,
    pub pipeline_step: String,
    pub category: String,
    pub annotation_engine: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: f64,
    pub attributes: Value,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_stack: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticEventInsert {
    pub event_id: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub run_id: Option<String>,
    pub file_hash: Option<String>,
    pub page_no: Option<u32>,
    pub timestamp: String,
    pub event_type: String,
    pub name: String,
    pub severity: String,
    pub message: String,
    pub attributes: Value,
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
include!("storage/replay.rs");
include!("storage/diagnostics_writes.rs");
include!("storage/diagnostics_queries.rs");
include!("storage/diagnostics_rows.rs");
include!("storage/helpers.rs");
