use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct HealthPayload {
    pub(crate) ok: bool,
    pub(crate) service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ShutdownRequest {
    pub(crate) confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ShutdownPayload {
    pub(crate) state: String,
    pub(crate) source: String,
    pub(crate) message: String,
    pub(crate) active_run_ids: Vec<String>,
    pub(crate) active_download_count: u32,
    pub(crate) grace_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct StatusPayload {
    pub(crate) state: String,
    pub(crate) host: String,
    pub(crate) active_run_id: Option<String>,
    pub(crate) default_profile: String,
    pub(crate) version: String,
    pub(crate) git_tag: String,
    pub(crate) git_sha: String,
    pub(crate) supported_inputs: Vec<String>,
    pub(crate) runtime_platform: Option<String>,
    pub(crate) accelerator: Option<String>,
    pub(crate) runtime_selectable: bool,
    pub(crate) runtime_variants: Vec<RuntimeVariantRecord>,
    pub(crate) inference_engine: String,
    pub(crate) log_path: String,
    pub(crate) database_path: String,
    pub(crate) duckdb_extensions: DuckDbExtensionsRecord,
    pub(crate) realtime_path: String,
    pub(crate) selected_model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct DuckDbExtensionsRecord {
    pub(crate) fts_loaded: bool,
    pub(crate) fts_error: Option<String>,
    pub(crate) vss_loaded: bool,
    pub(crate) vss_error: Option<String>,
    pub(crate) duckpgq_loaded: bool,
    pub(crate) duckpgq_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct RuntimeVariantRecord {
    pub(crate) runtime_id: String,
    pub(crate) label: String,
    pub(crate) platform: String,
    pub(crate) accelerator: String,
    pub(crate) backend: String,
    pub(crate) ffi_library: String,
    pub(crate) installed: bool,
    pub(crate) hardware_supported: bool,
    pub(crate) selectable: bool,
    pub(crate) selected: bool,
    pub(crate) support_detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct OcrProfileRecord {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) engine_name: String,
    pub(crate) description: String,
    pub(crate) default_max_tokens: u32,
    pub(crate) ngram_size: u32,
    pub(crate) ngram_window: u32,
    pub(crate) pdf_ngram_window: u32,
    pub(crate) force_prompt_eos: bool,
    pub(crate) no_image_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelAssetRecord {
    pub(crate) model_id: String,
    pub(crate) display_name: String,
    pub(crate) model_kind: String,
    pub(crate) routing_origin: String,
    pub(crate) status: String,
    pub(crate) repo_id: String,
    pub(crate) revision: String,
    pub(crate) local_path: Option<String>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) error: Option<String>,
    pub(crate) error_kind: Option<String>,
    pub(crate) model_file: String,
    pub(crate) mmproj_file: String,
    pub(crate) current_file: Option<String>,
    pub(crate) status_message: Option<String>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
    pub(crate) overall_downloaded_bytes: u64,
    pub(crate) overall_total_bytes: Option<u64>,
    pub(crate) overall_percent: f64,
    pub(crate) bytes_per_second: f64,
    pub(crate) eta_seconds: Option<u64>,
    pub(crate) auth_available: bool,
    pub(crate) auth_source: Option<String>,
    pub(crate) last_event_at: Option<String>,
    pub(crate) files: Vec<ModelDownloadFileRecord>,
    pub(crate) quantization: String,
    pub(crate) bits: u8,
    pub(crate) quality: String,
    pub(crate) hardware_tier: String,
    pub(crate) notes: String,
    pub(crate) recommended: bool,
    pub(crate) selected: bool,
    pub(crate) provider_name: String,
    pub(crate) embedding_dimension: Option<u32>,
    pub(crate) context_tokens: Option<u32>,
    pub(crate) pooling: Option<String>,
    pub(crate) normalize_embeddings: bool,
    pub(crate) query_prefix: Option<String>,
    pub(crate) document_prefix: Option<String>,
    pub(crate) recommended_vram_gb: Option<f64>,
    pub(crate) total_required_bytes: Option<u64>,
    pub(crate) downloaded_file_count: u32,
    pub(crate) total_file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelDownloadEvent {
    #[serde(flatten)]
    #[schema(value_type = ModelAssetRecord)]
    pub(crate) model: ModelAssetRecord,
    pub(crate) phase: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelDownloadFileRecord {
    pub(crate) file_id: String,
    pub(crate) file_name: String,
    pub(crate) status: String,
    pub(crate) local_path: Option<String>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
    pub(crate) percent: f64,
    pub(crate) bytes_per_second: f64,
    pub(crate) eta_seconds: Option<u64>,
    pub(crate) error: Option<String>,
    pub(crate) error_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelsPayload {
    pub(crate) models: Vec<ModelAssetRecord>,
    pub(crate) profiles: Vec<OcrProfileRecord>,
    pub(crate) selected_model_id: String,
    pub(crate) provider_repo: String,
    pub(crate) provider_label: String,
    pub(crate) shared_mmproj_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelDownloadRecord {
    pub(crate) model_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct ModelSelectRecord {
    pub(crate) model_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct ModelDownloadRequest {
    pub(crate) force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct SettingsPayload {
    pub(crate) pdf_dpi: u32,
    pub(crate) ocr_concurrency: u32,
    pub(crate) download_concurrency: u32,
    pub(crate) default_profile: String,
    pub(crate) retry_profile: String,
    pub(crate) cache_path: String,
    pub(crate) database_path: String,
    pub(crate) selected_runtime_id: String,
    pub(crate) selected_accelerator: String,
    pub(crate) selected_model_id: String,
    pub(crate) runtime_variants: Vec<RuntimeVariantRecord>,
    pub(crate) workbench_ui: WorkbenchUiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct SettingsUpdateRequest {
    pub(crate) default_profile: Option<String>,
    pub(crate) download_concurrency: Option<u32>,
    pub(crate) selected_runtime_id: Option<String>,
    pub(crate) workbench_ui: Option<WorkbenchUiSettingsPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct WorkbenchUiSettings {
    pub(crate) auto_follow_regions: bool,
    pub(crate) labels_visible: bool,
    pub(crate) overlay_visible: bool,
    pub(crate) panes_collapsed: WorkbenchPaneSettings,
    pub(crate) theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub(crate) struct WorkbenchPaneSettings {
    pub(crate) details: bool,
    pub(crate) diagnostics: bool,
    pub(crate) explorer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct WorkbenchUiSettingsPatch {
    pub(crate) auto_follow_regions: Option<bool>,
    pub(crate) labels_visible: Option<bool>,
    pub(crate) overlay_visible: Option<bool>,
    pub(crate) panes_collapsed: Option<WorkbenchPaneSettingsPatch>,
    pub(crate) theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub(crate) struct WorkbenchPaneSettingsPatch {
    pub(crate) details: Option<bool>,
    pub(crate) diagnostics: Option<bool>,
    pub(crate) explorer: Option<bool>,
}

impl Default for WorkbenchUiSettings {
    fn default() -> Self {
        Self {
            auto_follow_regions: true,
            labels_visible: true,
            overlay_visible: true,
            panes_collapsed: WorkbenchPaneSettings {
                details: true,
                diagnostics: true,
                explorer: false,
            },
            theme: "dark".to_string(),
        }
    }
}
