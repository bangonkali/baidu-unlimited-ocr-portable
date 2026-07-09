#![allow(dead_code)]

#[path = "openapi_modifiers.rs"]
mod openapi_modifiers;

use openapi_modifiers::BinaryImageResponse;
use utoipa::OpenApi;

use crate::{
    error::ErrorPayload,
    types::{
        DuckDbExtensionsRecord, HealthPayload, ModelAssetRecord, ModelDownloadEvent,
        ModelDownloadFileRecord, ModelDownloadRecord, ModelDownloadRequest, ModelSelectRecord,
        ModelsPayload, OcrProfileRecord, RuntimeVariantRecord, SettingsPayload,
        SettingsUpdateRequest, ShutdownPayload, ShutdownRequest, StatusPayload,
        WorkbenchPaneSettings, WorkbenchPaneSettingsPatch, WorkbenchUiSettings,
        WorkbenchUiSettingsPatch,
    },
    workbench_types::{
        DiagnosticAnalyticsPayload, DiagnosticAnalyticsSummary, DiagnosticBreakdownRecord,
        DiagnosticEventRecord, DiagnosticModelLeaseRecord, DiagnosticModelsPayload,
        DiagnosticPipelineTaskRecord, DiagnosticProgressPayload, DiagnosticProgressSummary,
        DiagnosticRecommendationRecord, DiagnosticRunRecord, DiagnosticRunsPayload,
        DiagnosticSlowSpanRecord, DiagnosticSpanRecord, DiagnosticTracePayload,
        DiagnosticTraceSummary, DiagnosticWaterfallPayload, DiagnosticWaterfallRowRecord,
        DiagnosticWaterfallSummary, DiagnosticWorkUnitDetailPayload, DiagnosticWorkUnitRecord,
        DocumentDetail, DocumentRegionsPayload, DocumentSummary, DocumentTextPayload,
        DocumentsPayload, FolderDialogResponse, GenerateEmbeddingRequest,
        GenerateEmbeddingResponse, HybridSearchFileResult, HybridSearchHit, HybridSearchRequest,
        HybridSearchResponse, IngestEngineConfigRecord, IngestEnginePresetRecord,
        IngestEngineSelection, IngestEnginesPayload, IngestPreviewResultRecord,
        IngestPreviewResultsPayload, IngestRunRecord, IngestRunsPayload, IngestStartRequest,
        IngestStartResponse, LogRecord, LogsPayload, OcrGeometry, OcrGeometryBounds,
        OcrGeometryPoint, OcrMetricsTreeNode, OcrMetricsTreePayload, OcrReplayPayload, OverlayBox,
        PageTextRecord, PipelineTaskRecord, PreviewImagesPayload, RealtimeEventRecord,
        RunCompletionManifestRecord, TextIndexRequest, TextIndexResponse, TextRegionSpan,
        UsedEmbeddingModelRecord, UsedEmbeddingModelsPayload,
    },
};

#[derive(Debug, OpenApi)]
#[openapi(
    info(title = "Trapo Server API", version = "0.1.5", description = "Rust Axum API for Trapo OCR workbench."),
    paths(
        health_doc, status_doc, openapi_doc, folder_dialog_doc, shutdown_doc,
        ingest_engines_doc, start_ingest_doc, list_runs_doc, get_run_doc, preview_results_doc,
        resume_run_doc, stop_run_doc, run_events_doc, ocr_events_doc,
        diagnostics_runs_doc, diagnostics_trace_doc, diagnostics_waterfall_doc,
        diagnostics_progress_doc, diagnostics_work_unit_detail_doc, diagnostics_analytics_doc,
        diagnostics_models_doc,
        start_text_index_doc, generate_embedding_doc, used_embedding_models_doc, hybrid_search_doc,
        run_metrics_doc, recent_metrics_doc, list_documents_doc, search_documents_doc,
        get_document_doc, document_regions_doc, region_snippet_doc, document_text_doc,
        preview_images_doc, preview_image_doc, settings_doc, update_settings_doc, models_doc,
        download_model_doc, select_model_doc, cancel_model_doc, model_events_doc,
        logs_doc, logs_export_doc
    ),
    components(schemas(
        ErrorPayload, HealthPayload, ShutdownRequest, ShutdownPayload, StatusPayload,
        DuckDbExtensionsRecord, RuntimeVariantRecord, OcrProfileRecord, ModelAssetRecord,
        ModelDownloadFileRecord, ModelDownloadRecord, ModelDownloadEvent, ModelSelectRecord,
        ModelDownloadRequest, ModelsPayload, SettingsPayload, SettingsUpdateRequest,
        WorkbenchUiSettings, WorkbenchPaneSettings, WorkbenchUiSettingsPatch,
        WorkbenchPaneSettingsPatch, IngestStartRequest, IngestStartResponse,
        IngestEngineSelection, IngestEngineConfigRecord, IngestEnginePresetRecord,
        IngestEnginesPayload, IngestPreviewResultRecord, IngestPreviewResultsPayload,
        RunCompletionManifestRecord, IngestRunRecord, IngestRunsPayload, PipelineTaskRecord,
        TextIndexRequest, TextIndexResponse, GenerateEmbeddingRequest, GenerateEmbeddingResponse,
        UsedEmbeddingModelsPayload, UsedEmbeddingModelRecord, HybridSearchRequest,
        HybridSearchResponse, HybridSearchFileResult, HybridSearchHit, OcrMetricsTreeNode,
        OcrMetricsTreePayload, DocumentSummary, DocumentDetail, DocumentsPayload,
        DocumentRegionsPayload, OverlayBox, OcrGeometry, OcrGeometryPoint, OcrGeometryBounds,
        TextRegionSpan, PageTextRecord, DocumentTextPayload, FolderDialogResponse,
        PreviewImagesPayload, LogRecord, LogsPayload, RealtimeEventRecord, OcrReplayPayload,
        DiagnosticRunRecord, DiagnosticRunsPayload, DiagnosticSpanRecord, DiagnosticEventRecord,
        DiagnosticTraceSummary, DiagnosticTracePayload, DiagnosticWaterfallRowRecord,
        DiagnosticWaterfallSummary, DiagnosticWaterfallPayload, DiagnosticWorkUnitRecord,
        DiagnosticModelLeaseRecord, DiagnosticPipelineTaskRecord, DiagnosticProgressSummary,
        DiagnosticProgressPayload, DiagnosticWorkUnitDetailPayload, DiagnosticBreakdownRecord,
        DiagnosticSlowSpanRecord, DiagnosticRecommendationRecord, DiagnosticAnalyticsSummary,
        DiagnosticAnalyticsPayload, DiagnosticModelsPayload
    )),
    tags(
        (name = "system"),
        (name = "ingest"),
        (name = "rag"),
        (name = "diagnostics"),
        (name = "documents"),
        (name = "settings"),
        (name = "models"),
        (name = "logs")
    ),
    modifiers(&BinaryImageResponse)
)]
/// `OpenAPI` document generator for the Trapo server API.
pub struct ApiDoc;

/// Builds the `OpenAPI` document with Trapo's post-generation schema fixes applied.
#[must_use]
pub fn openapi_document() -> utoipa::openapi::OpenApi {
    let mut document = ApiDoc::openapi();
    openapi_modifiers::apply_openapi_modifiers(&mut document);
    document
}

#[utoipa::path(get, path = "/api/health", tag = "system", responses((status = 200, body = HealthPayload)))]
const fn health_doc() {}

#[utoipa::path(get, path = "/api/status", tag = "system", responses((status = 200, body = StatusPayload)))]
const fn status_doc() {}

#[utoipa::path(get, path = "/api/openapi.json", tag = "system", responses((status = 200, description = "OpenAPI JSON")))]
const fn openapi_doc() {}

#[utoipa::path(post, path = "/api/system/folder-dialog", tag = "system", responses((status = 200, body = FolderDialogResponse)))]
const fn folder_dialog_doc() {}

#[utoipa::path(post, path = "/api/system/shutdown", tag = "system", request_body = ShutdownRequest, responses((status = 202, description = "Clean shutdown requested", body = ShutdownPayload), (status = 400, body = ErrorPayload)))]
const fn shutdown_doc() {}

#[utoipa::path(post, path = "/api/ingest/start", tag = "ingest", request_body = IngestStartRequest, responses((status = 202, description = "Ingest run queued", body = IngestStartResponse), (status = 400, body = ErrorPayload)))]
const fn start_ingest_doc() {}

#[utoipa::path(get, path = "/api/ingest/engines", tag = "ingest", responses((status = 200, body = IngestEnginesPayload)))]
const fn ingest_engines_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs", tag = "ingest", responses((status = 200, body = IngestRunsPayload)))]
const fn list_runs_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
const fn get_run_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/preview-results", tag = "ingest", params(("run_id" = String, Path), ("file_hash" = String, Query)), responses((status = 200, body = IngestPreviewResultsPayload), (status = 404, body = ErrorPayload)))]
const fn preview_results_doc() {}

#[utoipa::path(post, path = "/api/ingest/runs/{run_id}/stop", tag = "ingest", params(("run_id" = String, Path)), responses((status = 202, description = "Stop requested", body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
const fn stop_run_doc() {}

#[utoipa::path(post, path = "/api/ingest/runs/{run_id}/resume", tag = "ingest", params(("run_id" = String, Path)), responses((status = 202, description = "Run re-queued for resume", body = IngestStartResponse), (status = 404, body = ErrorPayload), (status = 409, body = ErrorPayload)))]
const fn resume_run_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/events", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, description = "Server-sent run snapshots", body = String, content_type = "text/event-stream"), (status = 404, body = ErrorPayload)))]
const fn run_events_doc() {}

#[utoipa::path(get, path = "/api/ocr/events", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("run_engine_id" = Option<String>, Query), ("file_hash" = Option<String>, Query), ("page_no" = Option<u32>, Query), ("since_sequence" = Option<u64>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = OcrReplayPayload)))]
const fn ocr_events_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/runs", tag = "diagnostics", params(("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticRunsPayload)))]
const fn diagnostics_runs_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/trace", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("file_hash" = Option<String>, Query), ("page_no" = Option<u32>, Query), ("status" = Option<String>, Query), ("q" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticTracePayload)))]
const fn diagnostics_trace_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/waterfall", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("file_hash" = Option<String>, Query), ("page_no" = Option<u32>, Query), ("status" = Option<String>, Query), ("q" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticWaterfallPayload)))]
const fn diagnostics_waterfall_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/progress", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticProgressPayload)))]
const fn diagnostics_progress_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/work-units/{work_unit_id}", tag = "diagnostics", params(("work_unit_id" = String, Path)), responses((status = 200, body = DiagnosticWorkUnitDetailPayload), (status = 404, body = ErrorPayload)))]
const fn diagnostics_work_unit_detail_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/analytics", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticAnalyticsPayload)))]
const fn diagnostics_analytics_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/models", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticModelsPayload)))]
const fn diagnostics_models_doc() {}

#[utoipa::path(post, path = "/api/rag/text-index", tag = "rag", request_body = TextIndexRequest, responses((status = 202, description = "Text index task completed or queued", body = TextIndexResponse), (status = 400, body = ErrorPayload), (status = 409, body = ErrorPayload)))]
const fn start_text_index_doc() {}

#[utoipa::path(post, path = "/api/rag/embeddings", tag = "rag", request_body = GenerateEmbeddingRequest, responses((status = 202, description = "Embedding generation task completed or queued", body = GenerateEmbeddingResponse), (status = 400, body = ErrorPayload), (status = 409, body = ErrorPayload)))]
const fn generate_embedding_doc() {}

#[utoipa::path(get, path = "/api/rag/embedding-models/used", tag = "rag", responses((status = 200, body = UsedEmbeddingModelsPayload)))]
const fn used_embedding_models_doc() {}

#[utoipa::path(post, path = "/api/rag/search", tag = "rag", request_body = HybridSearchRequest, responses((status = 200, body = HybridSearchResponse), (status = 400, body = ErrorPayload)))]
const fn hybrid_search_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/metrics", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, body = OcrMetricsTreePayload)))]
const fn run_metrics_doc() {}

#[utoipa::path(get, path = "/api/ingest/metrics/recent", tag = "ingest", params(("limit" = Option<u32>, Query)), responses((status = 200, body = OcrMetricsTreePayload)))]
const fn recent_metrics_doc() {}

#[utoipa::path(get, path = "/api/documents", tag = "documents", params(("q" = Option<String>, Query)), responses((status = 200, body = DocumentsPayload)))]
const fn list_documents_doc() {}

#[utoipa::path(get, path = "/api/search", tag = "documents", params(("q" = Option<String>, Query)), responses((status = 200, description = "DuckDB-backed document text search", body = DocumentsPayload)))]
const fn search_documents_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentDetail), (status = 404, body = ErrorPayload)))]
const fn get_document_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/regions", tag = "documents", params(("file_hash" = String, Path), ("run_id" = Option<String>, Query), ("run" = Option<String>, Query), ("run_engine_id" = Option<String>, Query), ("result" = Option<String>, Query)), responses((status = 200, body = DocumentRegionsPayload)))]
const fn document_regions_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/regions/{region_id}/snippet", tag = "documents", params(("file_hash" = String, Path), ("region_id" = String, Path)), responses((status = 200, description = "Region image snippet bytes", content((String = "image/png"))), (status = 404, body = ErrorPayload)))]
const fn region_snippet_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/text", tag = "documents", params(("file_hash" = String, Path), ("run_id" = Option<String>, Query), ("run" = Option<String>, Query), ("run_engine_id" = Option<String>, Query), ("result" = Option<String>, Query)), responses((status = 200, body = DocumentTextPayload)))]
const fn document_text_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/preview-images", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = PreviewImagesPayload)))]
const fn preview_images_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/preview-images/{variant}/{page_no}", tag = "documents", params(("file_hash" = String, Path), ("variant" = String, Path), ("page_no" = u32, Path)), responses((status = 200, description = "Preview image bytes", content((String = "image/png"), (String = "image/jpeg"))), (status = 404, body = ErrorPayload)))]
const fn preview_image_doc() {}

#[utoipa::path(get, path = "/api/settings", tag = "settings", responses((status = 200, body = SettingsPayload)))]
const fn settings_doc() {}

#[utoipa::path(put, path = "/api/settings", tag = "settings", request_body = SettingsUpdateRequest, responses((status = 200, body = SettingsPayload), (status = 400, body = ErrorPayload)))]
const fn update_settings_doc() {}

#[utoipa::path(get, path = "/api/models", tag = "models", responses((status = 200, body = ModelsPayload)))]
const fn models_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/download", tag = "models", params(("model_id" = String, Path)), request_body = ModelDownloadRequest, responses((status = 202, body = ModelDownloadRecord), (status = 409, body = ErrorPayload)))]
const fn download_model_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/select", tag = "models", params(("model_id" = String, Path)), responses((status = 202, description = "Model selected for subsequent ingest runs", body = ModelSelectRecord), (status = 400, body = ErrorPayload)))]
const fn select_model_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/cancel", tag = "models", params(("model_id" = String, Path)), responses((status = 202, description = "Download cancellation requested", body = ModelDownloadRecord), (status = 400, body = ErrorPayload)))]
const fn cancel_model_doc() {}

#[utoipa::path(get, path = "/api/models/{model_id}/events", tag = "models", params(("model_id" = String, Path)), responses((status = 200, description = "Server-sent model download progress events", body = String, content_type = "text/event-stream"), (status = 400, body = ErrorPayload)))]
const fn model_events_doc() {}

#[utoipa::path(get, path = "/api/logs/recent", tag = "logs", params(("limit" = Option<u32>, Query), ("level" = Option<String>, Query), ("component" = Option<String>, Query), ("q" = Option<String>, Query)), responses((status = 200, body = LogsPayload)))]
const fn logs_doc() {}

#[utoipa::path(get, path = "/api/logs/export", tag = "logs", responses((status = 200, description = "Plain text log export", body = String, content_type = "text/plain")))]
const fn logs_export_doc() {}
