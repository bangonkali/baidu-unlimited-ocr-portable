use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub(crate) use crate::ocr_geometry::{OcrGeometry, OcrGeometryBounds, OcrGeometryPoint};
pub(crate) use crate::workbench_diagnostics_types::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct IngestStartRequest {
    pub(crate) root_path: String,
    pub(crate) profile_id: Option<String>,
    pub(crate) model_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    pub(crate) engine_id: Option<String>,
    pub(crate) engines: Option<Vec<IngestEngineSelection>>,
    pub(crate) reprocess: Option<bool>,
    pub(crate) text_index_after_ingest: Option<bool>,
    pub(crate) embedding_after_ingest: Option<bool>,
    pub(crate) embedding_model_id: Option<String>,
    pub(crate) embedding_dimension: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct IngestEngineSelection {
    pub(crate) preset_id: Option<String>,
    pub(crate) engine_id: String,
    pub(crate) engine_kind: String,
    pub(crate) model_id: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    #[schema(value_type = Object)]
    pub(crate) parameters: Option<Value>,
    pub(crate) ordinal: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestEngineConfigRecord {
    pub(crate) run_engine_id: String,
    pub(crate) run_id: String,
    pub(crate) ordinal: u32,
    pub(crate) engine_kind: String,
    pub(crate) engine_id: String,
    pub(crate) label: String,
    pub(crate) model_id: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    #[schema(value_type = Object)]
    pub(crate) parameters: Value,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
    pub(crate) usable_output_count: u32,
    pub(crate) previewer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestEnginePresetRecord {
    pub(crate) preset_id: String,
    pub(crate) engine_id: String,
    pub(crate) engine_kind: String,
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) model_id: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    pub(crate) previewer: String,
    pub(crate) default_enabled: bool,
    pub(crate) requires_model: bool,
    pub(crate) download_model_ids: Vec<String>,
    pub(crate) available: bool,
    pub(crate) availability: String,
    pub(crate) availability_detail: Option<String>,
    pub(crate) runner_kind: String,
    pub(crate) runner_status: String,
    pub(crate) runner_detail: Option<String>,
    #[schema(value_type = Object)]
    pub(crate) parameter_schema: Value,
    #[schema(value_type = Object)]
    pub(crate) default_parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestEnginesPayload {
    pub(crate) engines: Vec<IngestEnginePresetRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestPreviewResultRecord {
    pub(crate) run_engine_id: String,
    pub(crate) run_id: String,
    pub(crate) ordinal: u32,
    pub(crate) engine_kind: String,
    pub(crate) engine_id: String,
    pub(crate) label: String,
    pub(crate) model_id: Option<String>,
    pub(crate) profile_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    pub(crate) status: String,
    pub(crate) previewer: String,
    pub(crate) output_count: u32,
    pub(crate) page_count: u32,
    pub(crate) error: Option<String>,
    pub(crate) runner_kind: String,
    pub(crate) runner_status: String,
    pub(crate) runner_detail: Option<String>,
    #[schema(value_type = Object)]
    pub(crate) provenance: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct IngestPreviewResultsPayload {
    pub(crate) run_id: String,
    pub(crate) file_hash: String,
    pub(crate) results: Vec<IngestPreviewResultRecord>,
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
    pub(crate) engine_configs: Vec<IngestEngineConfigRecord>,
    pub(crate) preview_results: Vec<IngestPreviewResultRecord>,
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
pub(crate) struct PipelineTaskRecord {
    pub(crate) task_id: String,
    pub(crate) task_kind: String,
    pub(crate) origin_run_id: Option<String>,
    pub(crate) status: String,
    #[schema(value_type = Object)]
    pub(crate) params: Value,
    #[schema(value_type = Object)]
    pub(crate) result: Value,
    pub(crate) queued_at: String,
    pub(crate) started_at: Option<String>,
    pub(crate) finished_at: Option<String>,
    pub(crate) runner_id: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct TextIndexRequest {
    pub(crate) source_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct TextIndexResponse {
    pub(crate) task: PipelineTaskRecord,
    pub(crate) text_index_run_id: String,
    pub(crate) source_run_id: String,
    pub(crate) segments_indexed: u32,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct GenerateEmbeddingRequest {
    pub(crate) source_run_id: String,
    pub(crate) model_id: String,
    pub(crate) dimension: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct GenerateEmbeddingResponse {
    pub(crate) task: PipelineTaskRecord,
    pub(crate) embedding_run_id: String,
    pub(crate) source_run_id: String,
    pub(crate) model_id: String,
    pub(crate) dimension: u32,
    pub(crate) segments_embedded: u32,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct UsedEmbeddingModelsPayload {
    pub(crate) models: Vec<UsedEmbeddingModelRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct UsedEmbeddingModelRecord {
    pub(crate) model_id: String,
    pub(crate) display_name: String,
    pub(crate) dimension: u32,
    pub(crate) provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct HybridSearchRequest {
    pub(crate) query: String,
    pub(crate) source_run_id: Option<String>,
    pub(crate) embedding_model_id: Option<String>,
    pub(crate) limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct HybridSearchResponse {
    pub(crate) query: String,
    pub(crate) hits: Vec<HybridSearchHit>,
    pub(crate) files: Vec<HybridSearchFileResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct HybridSearchFileResult {
    pub(crate) file_hash: String,
    pub(crate) hit_count: u32,
    pub(crate) relevance_score: f64,
    pub(crate) hits: Vec<HybridSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct HybridSearchHit {
    pub(crate) segment_id: String,
    pub(crate) file_hash: String,
    pub(crate) page_no: u32,
    pub(crate) annotation_id: Option<String>,
    pub(crate) category: String,
    pub(crate) text: String,
    pub(crate) score: f64,
    pub(crate) relevance_score: f64,
    pub(crate) rank: u32,
    pub(crate) hit_source: String,
    pub(crate) model_id: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_engine_id: Option<String>,
    pub(crate) boxes: Vec<OverlayBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OverlayBox {
    pub(crate) region_id: String,
    #[schema(value_type = String)]
    pub(crate) annotation_id: String,
    pub(crate) label: String,
    pub(crate) category: String,
    pub(crate) content_markdown: String,
    pub(crate) content_html: Option<String>,
    pub(crate) page_no: u32,
    pub(crate) left_percent: f64,
    pub(crate) top_percent: f64,
    pub(crate) width_percent: f64,
    pub(crate) height_percent: f64,
    pub(crate) hidden: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<OcrGeometry>, nullable = true)]
    pub(crate) geometry: Option<OcrGeometry>,
    #[serde(skip)]
    #[schema(ignore)]
    pub(crate) source_region_key: String,
}

impl OverlayBox {
    pub(crate) fn resolved_geometry(&self) -> OcrGeometry {
        self.geometry.clone().unwrap_or_else(|| {
            OcrGeometry::axis_aligned(
                self.left_percent,
                self.top_percent,
                self.width_percent,
                self.height_percent,
            )
        })
    }

    pub(crate) fn storage_bbox_kind(&self) -> String {
        self.resolved_geometry().kind
    }

    pub(crate) fn storage_geometry_json(&self) -> String {
        let geometry = self.resolved_geometry();
        serde_json::to_string(&geometry).unwrap_or_else(|_| "{}".to_string())
    }

    pub(crate) fn storage_coordinate_space(&self) -> String {
        self.resolved_geometry().coordinate_space
    }

    pub(crate) fn storage_rotation_degrees(&self) -> Option<f64> {
        self.resolved_geometry().rotation_degrees
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_engine_id: Option<String>,
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
