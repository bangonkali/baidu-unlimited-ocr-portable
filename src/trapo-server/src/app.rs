use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    catalog::{
        DEFAULT_MODEL_ID, DEFAULT_PROFILE_ID, PROVIDER_LABEL, PROVIDER_REPO_ID, PROVIDER_REVISION,
        RETRY_PROFILE_ID, SHARED_MMPROJ_FILE, SHARED_MMPROJ_SIZE_BYTES, choose_runtime_id,
        find_model, find_profile, model_catalog, ocr_profiles, runtime_record, runtime_variants,
    },
    config::ServerConfig,
    error::{AppError, Result},
    logger::AppLogger,
    pdf::{PdfRenderer, is_pdf},
    realtime::RealtimeHub,
    routes,
    scanner::{
        DiscoveredFile, SUPPORTED_INPUTS, discover_supported_files, generic_path, stable_hash,
    },
    storage::{OcrPageMetrics, Repository, StoredDocument, StoredPage, StoredRun},
    types::{
        HealthPayload, ModelAssetRecord, ModelDownloadEvent, ModelDownloadFileRecord,
        ModelDownloadRecord, ModelDownloadRequest, ModelSelectRecord, ModelsPayload,
        SettingsPayload, SettingsUpdateRequest, StatusPayload, WorkbenchUiSettings,
    },
    workbench_types::{
        DocumentDetail, DocumentRegionsPayload, DocumentSummary, DocumentTextPayload,
        DocumentsPayload, FolderDialogResponse, IngestRunRecord, IngestRunsPayload,
        IngestStartRequest, LogsPayload, OcrMetricsTreeNode, OcrMetricsTreePayload, PageTextRecord,
        PreviewImagesPayload,
    },
};

const ENGINE_ID: &str = "unlimited-ocr-ffi";
const PDF_DPI: u32 = 200;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppInner>,
}

#[derive(Debug)]
struct AppInner {
    config: ServerConfig,
    repository: Repository,
    logger: AppLogger,
    hub: Arc<RealtimeHub>,
    renderer: PdfRenderer,
    state: Mutex<WorkbenchState>,
}

#[derive(Debug)]
struct WorkbenchState {
    selected_model_id: String,
    selected_profile_id: String,
    selected_runtime_id: String,
    runtime_variants: Vec<crate::catalog::RuntimeVariant>,
    workbench_ui: WorkbenchUiSettings,
    active_run_id: Option<String>,
    runs: BTreeMap<String, RunState>,
    documents: BTreeMap<String, DocumentState>,
    downloads: HashMap<String, DownloadState>,
}

#[derive(Debug, Clone)]
pub struct RunState {
    pub run_id: String,
    pub root_path: String,
    pub status: String,
    pub queued_files: u32,
    pub processed_pages: u32,
    pub total_pages: u32,
    pub current_page: Option<u32>,
    pub profile_id: String,
    pub engine_id: String,
    pub model_id: String,
    pub runtime_id: String,
    pub error: Option<String>,
    pub cancel_requested: bool,
    pub file_hashes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DocumentState {
    pub file_hash: String,
    pub display_name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub root_path: PathBuf,
    pub status: String,
    pub page_count: u32,
    pub error: Option<String>,
    pub pages: Vec<PageState>,
}

#[derive(Debug, Clone)]
pub struct PageState {
    pub page_no: u32,
    pub image_path: PathBuf,
    pub width_px: u32,
    pub height_px: u32,
    pub render_dpi: u32,
    pub status: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub boxes: Vec<crate::workbench_types::OverlayBox>,
    pub spans: Vec<crate::workbench_types::TextRegionSpan>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
struct DownloadState {
    status: String,
    current_file: Option<String>,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    error: Option<String>,
    started_at: Option<Instant>,
    cancel_requested: bool,
    last_event_at: Option<String>,
}

pub fn build_router(state: AppState) -> axum::Router {
    routes::router(state)
}

include!("app/core.rs");
include!("app/model_methods.rs");
include!("app/ingest_start.rs");
include!("app/run_document_methods.rs");
include!("app/download_runtime.rs");
include!("app/ingest_pipeline.rs");
include!("app/page_pipeline.rs");
include!("app/download_helpers.rs");
include!("app/logging.rs");

include!("app/settings_helpers.rs");
include!("app/model_records.rs");
include!("app/document_records.rs");

#[cfg(test)]
include!("app/tests.rs");
