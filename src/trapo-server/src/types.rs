use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthPayload {
    pub ok: bool,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatusPayload {
    pub state: String,
    pub host: String,
    pub active_run_id: Option<String>,
    pub default_profile: String,
    pub version: String,
    pub git_tag: String,
    pub git_sha: String,
    pub supported_inputs: Vec<String>,
    pub runtime_platform: Option<String>,
    pub accelerator: Option<String>,
    pub runtime_selectable: bool,
    pub runtime_variants: Vec<RuntimeVariantRecord>,
    pub inference_engine: String,
    pub log_path: String,
    pub database_path: String,
    pub realtime_path: String,
    pub selected_model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RuntimeVariantRecord {
    pub runtime_id: String,
    pub label: String,
    pub platform: String,
    pub accelerator: String,
    pub backend: String,
    pub ffi_library: String,
    pub installed: bool,
    pub hardware_supported: bool,
    pub selectable: bool,
    pub selected: bool,
    pub support_detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OcrProfileRecord {
    pub key: String,
    pub label: String,
    pub engine_name: String,
    pub description: String,
    pub default_max_tokens: u32,
    pub ngram_size: u32,
    pub ngram_window: u32,
    pub pdf_ngram_window: u32,
    pub force_prompt_eos: bool,
    pub no_image_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelAssetRecord {
    pub model_id: String,
    pub display_name: String,
    pub status: String,
    pub repo_id: String,
    pub revision: String,
    pub local_path: Option<String>,
    pub size_bytes: Option<u64>,
    pub error: Option<String>,
    pub model_file: String,
    pub mmproj_file: String,
    pub current_file: Option<String>,
    pub status_message: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub overall_downloaded_bytes: u64,
    pub overall_total_bytes: Option<u64>,
    pub overall_percent: f64,
    pub bytes_per_second: f64,
    pub eta_seconds: Option<u64>,
    pub auth_available: bool,
    pub auth_source: Option<String>,
    pub last_event_at: Option<String>,
    pub files: Vec<ModelDownloadFileRecord>,
    pub quantization: String,
    pub bits: u8,
    pub quality: String,
    pub hardware_tier: String,
    pub notes: String,
    pub recommended: bool,
    pub selected: bool,
    pub provider_name: String,
    pub total_required_bytes: Option<u64>,
    pub downloaded_file_count: u32,
    pub total_file_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelDownloadFileRecord {
    pub file_id: String,
    pub file_name: String,
    pub status: String,
    pub local_path: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percent: f64,
    pub bytes_per_second: f64,
    pub eta_seconds: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelsPayload {
    pub models: Vec<ModelAssetRecord>,
    pub profiles: Vec<OcrProfileRecord>,
    pub selected_model_id: String,
    pub provider_repo: String,
    pub provider_label: String,
    pub shared_mmproj_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelDownloadRecord {
    pub model_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModelSelectRecord {
    pub model_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct ModelDownloadRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SettingsPayload {
    pub pdf_dpi: u32,
    pub ocr_concurrency: u32,
    pub default_profile: String,
    pub retry_profile: String,
    pub cache_path: String,
    pub database_path: String,
    pub selected_runtime_id: String,
    pub selected_accelerator: String,
    pub selected_model_id: String,
    pub runtime_variants: Vec<RuntimeVariantRecord>,
    pub workbench_ui: WorkbenchUiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct SettingsUpdateRequest {
    pub default_profile: Option<String>,
    pub selected_runtime_id: Option<String>,
    pub workbench_ui: Option<WorkbenchUiSettingsPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkbenchUiSettings {
    pub auto_follow_regions: bool,
    pub labels_visible: bool,
    pub overlay_visible: bool,
    pub panes_collapsed: WorkbenchPaneSettings,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkbenchPaneSettings {
    pub details: bool,
    pub diagnostics: bool,
    pub explorer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct WorkbenchUiSettingsPatch {
    pub auto_follow_regions: Option<bool>,
    pub labels_visible: Option<bool>,
    pub overlay_visible: Option<bool>,
    pub panes_collapsed: Option<WorkbenchPaneSettingsPatch>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct WorkbenchPaneSettingsPatch {
    pub details: Option<bool>,
    pub diagnostics: Option<bool>,
    pub explorer: Option<bool>,
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
