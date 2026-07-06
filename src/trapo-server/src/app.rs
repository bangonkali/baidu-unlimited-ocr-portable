use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use chrono::Utc;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    catalog::{
        DEFAULT_MODEL_ID, DEFAULT_PROFILE_ID, PROVIDER_LABEL, PROVIDER_REPO_ID, RETRY_PROFILE_ID,
        SHARED_MMPROJ_FILE, choose_runtime_id, embedding_model_catalog, find_model, find_profile,
        model_catalog, ocr_profiles, runtime_record, runtime_variants,
    },
    config::ServerConfig,
    embedding::{EmbeddingPurpose, generate_embeddings, profile_from_model_row},
    error::{AppError, Result},
    ids::new_persistence_id,
    logger::AppLogger,
    pdf::{PdfRenderer, RenderedPage, is_pdf},
    realtime::RealtimeHub,
    routes,
    scanner::{
        DiscoveredFile, SUPPORTED_INPUTS, discover_supported_files, generic_path, stable_hash,
    },
    shutdown::{BackgroundTasks, ShutdownCoordinator},
    storage::{
        AnnotationIdentityDraft, CompletedRunPage, DbExtensionCapabilities, DiagnosticEventInsert,
        DiagnosticEventRow, DiagnosticModelLeaseInsert, DiagnosticModelLeaseRow, DiagnosticRunRow,
        DiagnosticSpanInsert, DiagnosticSpanRow, DiagnosticTraceFilter, DiagnosticWorkUnitRow,
        DownloadEventInsert, OcrPageMetrics, PipelineTaskRow, RagEmbeddingModelRow,
        RagEmbeddingRunRow, RagEmbeddingVectorRow, RagSearchHitRow, RagTextIndexRunRow,
        RagTextSegmentRow, Repository, StoredDocument, StoredPage, StoredRealtimeEvent, StoredRun,
        StoredRunCompletionManifest, WorkUnitUpsert,
    },
    types::{
        DuckDbExtensionsRecord, HealthPayload, ModelAssetRecord, ModelDownloadEvent,
        ModelDownloadFileRecord, ModelDownloadRecord, ModelDownloadRequest, ModelSelectRecord,
        ModelsPayload, SettingsPayload, SettingsUpdateRequest, ShutdownPayload, StatusPayload,
        WorkbenchUiSettings,
    },
    workbench_types::{
        DiagnosticAnalyticsPayload, DiagnosticAnalyticsSummary, DiagnosticBreakdownRecord,
        DiagnosticEventRecord, DiagnosticModelLeaseRecord, DiagnosticModelsPayload,
        DiagnosticPipelineTaskRecord, DiagnosticProgressPayload, DiagnosticProgressSummary,
        DiagnosticRecommendationRecord, DiagnosticRunRecord, DiagnosticRunsPayload,
        DiagnosticSlowSpanRecord, DiagnosticSpanRecord, DiagnosticTracePayload,
        DiagnosticTraceSummary, DiagnosticWorkUnitRecord, DocumentDetail, DocumentRegionsPayload,
        DocumentSummary, DocumentTextPayload, DocumentsPayload, FolderDialogResponse,
        GenerateEmbeddingRequest, GenerateEmbeddingResponse, HybridSearchFileResult,
        HybridSearchHit, HybridSearchRequest, HybridSearchResponse, IngestRunRecord,
        IngestRunsPayload, IngestStartRequest, IngestStartResponse, LogsPayload,
        OcrMetricsTreeNode, OcrMetricsTreePayload, OcrReplayPayload, PageTextRecord,
        PipelineTaskRecord, PreviewImagesPayload, RealtimeEventRecord, RunCompletionManifestRecord,
        TextIndexRequest, TextIndexResponse, UsedEmbeddingModelRecord, UsedEmbeddingModelsPayload,
    },
};

const ENGINE_ID: &str = "unlimited-ocr-ffi";
const PDF_DPI: u32 = 200;

/// Shared server state used by the Axum router.
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
    annotation_identities: AnnotationIdentityRuntime,
    background_tasks: BackgroundTasks,
    shutdown: ShutdownCoordinator,
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
    download_queue: VecDeque<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RunState {
    pub(crate) run_id: String,
    pub(crate) root_path: String,
    pub(crate) status: String,
    pub(crate) queued_files: u32,
    pub(crate) processed_pages: u32,
    pub(crate) total_pages: u32,
    pub(crate) current_page: Option<u32>,
    pub(crate) profile_id: String,
    pub(crate) engine_id: String,
    pub(crate) model_id: String,
    pub(crate) runtime_id: String,
    pub(crate) error: Option<String>,
    pub(crate) cancel_requested: bool,
    pub(crate) file_hashes: Vec<String>,
    pub(crate) completion_manifest: Option<StoredRunCompletionManifest>,
}

#[derive(Debug, Clone)]
pub(crate) struct DocumentState {
    pub(crate) file_hash: String,
    pub(crate) display_name: String,
    pub(crate) extension: String,
    pub(crate) size_bytes: u64,
    pub(crate) absolute_path: PathBuf,
    pub(crate) relative_path: PathBuf,
    pub(crate) root_path: PathBuf,
    pub(crate) status: String,
    pub(crate) page_count: u32,
    pub(crate) error: Option<String>,
    pub(crate) pages: Vec<PageState>,
}

#[derive(Debug, Clone)]
pub(crate) struct PageState {
    pub(crate) page_no: u32,
    pub(crate) image_path: PathBuf,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    pub(crate) render_dpi: u32,
    pub(crate) status: String,
    pub(crate) raw_text: String,
    pub(crate) cleaned_text: String,
    pub(crate) boxes: Vec<crate::workbench_types::OverlayBox>,
    pub(crate) spans: Vec<crate::workbench_types::TextRegionSpan>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone)]
struct DownloadState {
    download_id: String,
    download_key: String,
    owner_kind: String,
    owner_id: String,
    file_id: String,
    file_name: String,
    source_url: String,
    target_path: PathBuf,
    force: bool,
    status: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    error: Option<String>,
    started_at: Option<Instant>,
    last_progress_publish_at: Option<Instant>,
    last_progress_publish_bytes: u64,
    cancel_requested: bool,
    last_event_at: Option<String>,
}

/// Builds the Axum router for the API and static workbench assets.
pub fn build_router(state: AppState) -> axum::Router {
    routes::router(state)
}

fn elapsed_millis_u64(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn usize_to_u64_saturating(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn u32_to_i32_saturating(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

include!("app/core.rs");
include!("app/download_targets.rs");
include!("app/model_methods.rs");
include!("app/ingest_start.rs");
include!("app/run_document_methods.rs");
include!("app/run_resume_methods.rs");
include!("app/download_runtime.rs");
include!("app/annotation_identity_runtime.rs");
include!("app/ocr_stream_events.rs");
include!("app/ocr_worker.rs");
include!("app/ingest_pipeline.rs");
include!("app/process_document.rs");
include!("app/process_document_records.rs");
include!("app/page_diagnostics.rs");
include!("app/region_snippets.rs");
include!("app/page_completion.rs");
include!("app/page_pipeline.rs");
include!("app/rag_methods.rs");
include!("app/rag_execution.rs");
include!("app/rag_text_segments.rs");
include!("app/rag_records.rs");
include!("app/download_helpers.rs");
include!("app/shutdown.rs");
include!("app/logging.rs");
include!("app/diagnostics_recording.rs");
include!("app/diagnostics_methods.rs");
include!("app/diagnostics_records.rs");
include!("app/replay_methods.rs");

include!("app/settings_helpers.rs");
include!("app/model_records.rs");
include!("app/document_records.rs");

#[cfg(test)]
include!("app/tests.rs");
#[cfg(test)]
include!("app/rag_text_segments_tests.rs");
