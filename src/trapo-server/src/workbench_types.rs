use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use crate::workbench_diagnostics_types::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct IngestStartRequest {
    pub root_path: String,
    pub profile_id: Option<String>,
    pub model_id: Option<String>,
    pub engine_id: Option<String>,
    pub reprocess: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestRunRecord {
    pub run_id: String,
    pub root_path: String,
    pub status: String,
    pub file_hashes: Vec<String>,
    pub queued_files: u32,
    pub processed_pages: u32,
    pub total_pages: u32,
    pub current_page: Option<u32>,
    pub progress_percent: f64,
    pub profile_id: String,
    pub engine_id: String,
    pub model_id: String,
    pub runtime_id: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestStartResponse {
    pub run: IngestRunRecord,
    pub documents: Vec<DocumentSummary>,
    pub replay_since_sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IngestRunsPayload {
    pub runs: Vec<IngestRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OcrMetricsTreePayload {
    pub roots: Vec<OcrMetricsTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OcrMetricsTreeNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String,
    pub token_count: u64,
    pub avg_tps: f64,
    pub elapsed_ms: u64,
    #[schema(no_recursion)]
    pub children: Vec<OcrMetricsTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentSummary {
    pub file_hash: String,
    pub display_name: String,
    pub relative_path: String,
    pub status: String,
    pub page_count: u32,
    pub processed_pages: u32,
    pub total_pages: u32,
    pub current_page: Option<u32>,
    pub progress_percent: f64,
    pub regions: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentDetail {
    pub file_hash: String,
    pub display_name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub status: String,
    pub page_count: u32,
    pub processed_pages: u32,
    pub total_pages: u32,
    pub current_page: Option<u32>,
    pub progress_percent: f64,
    pub regions: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentsPayload {
    pub documents: Vec<DocumentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentRegionsPayload {
    pub file_hash: String,
    pub boxes: Vec<OverlayBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OverlayBox {
    pub region_id: String,
    pub label: String,
    pub content_markdown: String,
    pub content_html: Option<String>,
    pub page_no: u32,
    pub left_percent: f64,
    pub top_percent: f64,
    pub width_percent: f64,
    pub height_percent: f64,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TextRegionSpan {
    pub region_id: String,
    pub page_no: u32,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageTextRecord {
    pub page_no: u32,
    pub text: String,
    pub spans: Vec<TextRegionSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentTextPayload {
    pub file_hash: String,
    pub pages: Vec<PageTextRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PreviewImagesPayload {
    pub file_hash: String,
    pub variants: Vec<String>,
    pub pages: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FolderDialogResponse {
    pub cancelled: bool,
    pub selected_path: String,
    pub manual_path_supported: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogRecord {
    pub timestamp: String,
    pub level: String,
    pub component: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LogsPayload {
    pub log_path: String,
    pub logs: Vec<LogRecord>,
}
