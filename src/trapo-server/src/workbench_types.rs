use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub(crate) use crate::workbench_diagnostics_types::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct IngestStartRequest {
    pub(crate) root_path: String,
    pub(crate) profile_id: Option<String>,
    pub(crate) model_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    pub(crate) engine_id: Option<String>,
    pub(crate) reprocess: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct RunCompletionManifestRecord {
    pub(crate) run_id: String,
    pub(crate) completed_at: String,
    pub(crate) status: String,
    pub(crate) root_path: String,
    pub(crate) profile_id: String,
    pub(crate) engine_id: String,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) queued_files: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) file_count: u32,
    pub(crate) page_count: u32,
    #[schema(value_type = Object)]
    pub(crate) summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestRunRecord {
    pub(crate) run_id: String,
    pub(crate) root_path: String,
    pub(crate) status: String,
    pub(crate) file_hashes: Vec<String>,
    pub(crate) queued_files: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) current_page: Option<u32>,
    pub(crate) progress_percent: f64,
    pub(crate) profile_id: String,
    pub(crate) engine_id: String,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) error: Option<String>,
    pub(crate) can_resume: bool,
    pub(crate) can_restart: bool,
    pub(crate) completion_manifest: Option<RunCompletionManifestRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestStartResponse {
    pub(crate) run: IngestRunRecord,
    pub(crate) documents: Vec<DocumentSummary>,
    pub(crate) replay_since_sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestRunsPayload {
    pub(crate) runs: Vec<IngestRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrMetricsTreePayload {
    pub(crate) roots: Vec<OcrMetricsTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrMetricsTreeNode {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) token_count: u64,
    pub(crate) avg_tps: f64,
    pub(crate) elapsed_ms: u64,
    #[schema(no_recursion)]
    pub(crate) children: Vec<Self>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DocumentSummary {
    pub(crate) file_hash: String,
    pub(crate) display_name: String,
    pub(crate) relative_path: String,
    pub(crate) status: String,
    pub(crate) page_count: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) current_page: Option<u32>,
    pub(crate) progress_percent: f64,
    pub(crate) regions: u32,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DocumentDetail {
    pub(crate) file_hash: String,
    pub(crate) display_name: String,
    pub(crate) relative_path: String,
    pub(crate) absolute_path: String,
    pub(crate) status: String,
    pub(crate) page_count: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) current_page: Option<u32>,
    pub(crate) progress_percent: f64,
    pub(crate) regions: u32,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DocumentsPayload {
    pub(crate) documents: Vec<DocumentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DocumentRegionsPayload {
    pub(crate) file_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
    pub(crate) boxes: Vec<OverlayBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OverlayBox {
    pub(crate) region_id: String,
    #[schema(value_type = String)]
    pub(crate) annotation_id: String,
    pub(crate) label: String,
    pub(crate) content_markdown: String,
    pub(crate) content_html: Option<String>,
    pub(crate) page_no: u32,
    pub(crate) left_percent: f64,
    pub(crate) top_percent: f64,
    pub(crate) width_percent: f64,
    pub(crate) height_percent: f64,
    pub(crate) hidden: bool,
    #[serde(skip)]
    #[schema(ignore)]
    pub(crate) source_region_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct TextRegionSpan {
    pub(crate) region_id: String,
    #[schema(value_type = String)]
    pub(crate) annotation_id: String,
    pub(crate) page_no: u32,
    pub(crate) start: u64,
    pub(crate) end: u64,
    #[serde(skip)]
    #[schema(ignore)]
    pub(crate) source_region_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct PageTextRecord {
    pub(crate) page_no: u32,
    pub(crate) text: String,
    pub(crate) spans: Vec<TextRegionSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DocumentTextPayload {
    pub(crate) file_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
    pub(crate) pages: Vec<PageTextRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct PreviewImagesPayload {
    pub(crate) file_hash: String,
    pub(crate) variants: Vec<String>,
    pub(crate) pages: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct FolderDialogResponse {
    pub(crate) cancelled: bool,
    pub(crate) selected_path: String,
    pub(crate) manual_path_supported: bool,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct LogRecord {
    pub(crate) timestamp: String,
    pub(crate) level: String,
    pub(crate) component: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct LogsPayload {
    pub(crate) log_path: String,
    pub(crate) logs: Vec<LogRecord>,
}
