#![allow(dead_code)]

use utoipa::{Modify, OpenApi};

use crate::{
    error::ErrorPayload,
    types::{
        HealthPayload, ModelAssetRecord, ModelDownloadEvent, ModelDownloadFileRecord,
        ModelDownloadRecord, ModelDownloadRequest, ModelSelectRecord, ModelsPayload,
        OcrProfileRecord, RuntimeVariantRecord, SettingsPayload, SettingsUpdateRequest,
        StatusPayload, WorkbenchPaneSettings, WorkbenchPaneSettingsPatch, WorkbenchUiSettings,
        WorkbenchUiSettingsPatch,
    },
    workbench_types::{
        DiagnosticAnalyticsPayload, DiagnosticAnalyticsSummary, DiagnosticBreakdownRecord,
        DiagnosticEventRecord, DiagnosticModelLeaseRecord, DiagnosticModelsPayload,
        DiagnosticProgressPayload, DiagnosticProgressSummary, DiagnosticRecommendationRecord,
        DiagnosticRunRecord, DiagnosticRunsPayload, DiagnosticSlowSpanRecord, DiagnosticSpanRecord,
        DiagnosticTracePayload, DiagnosticTraceSummary, DiagnosticWorkUnitRecord, DocumentDetail,
        DocumentRegionsPayload, DocumentSummary, DocumentTextPayload, DocumentsPayload,
        FolderDialogResponse, IngestRunRecord, IngestRunsPayload, IngestStartRequest,
        IngestStartResponse, LogRecord, LogsPayload, OcrMetricsTreeNode, OcrMetricsTreePayload,
        OcrReplayPayload, OverlayBox, PageTextRecord, PreviewImagesPayload, RealtimeEventRecord,
        TextRegionSpan,
    },
};

#[derive(Debug, OpenApi)]
#[openapi(
    info(title = "Trapo Server API", version = "0.1.5", description = "Rust Axum API for Trapo OCR workbench."),
    paths(
        health_doc,
        status_doc,
        openapi_doc,
        folder_dialog_doc,
        start_ingest_doc,
        list_runs_doc,
        get_run_doc,
        stop_run_doc,
        run_events_doc,
        ocr_events_doc,
        diagnostics_runs_doc,
        diagnostics_trace_doc,
        diagnostics_progress_doc,
        diagnostics_analytics_doc,
        diagnostics_models_doc,
        run_metrics_doc,
        recent_metrics_doc,
        list_documents_doc,
        search_documents_doc,
        get_document_doc,
        document_regions_doc,
        region_snippet_doc,
        document_text_doc,
        preview_images_doc,
        preview_image_doc,
        settings_doc,
        update_settings_doc,
        models_doc,
        download_model_doc,
        select_model_doc,
        cancel_model_doc,
        model_events_doc,
        logs_doc
    ),
    components(schemas(
        ErrorPayload,
        HealthPayload,
        StatusPayload,
        RuntimeVariantRecord,
        OcrProfileRecord,
        ModelAssetRecord,
        ModelDownloadFileRecord,
        ModelDownloadRecord,
        ModelDownloadEvent,
        ModelSelectRecord,
        ModelDownloadRequest,
        ModelsPayload,
        SettingsPayload,
        SettingsUpdateRequest,
        WorkbenchUiSettings,
        WorkbenchPaneSettings,
        WorkbenchUiSettingsPatch,
        WorkbenchPaneSettingsPatch,
        IngestStartRequest,
        IngestStartResponse,
        IngestRunRecord,
        IngestRunsPayload,
        OcrMetricsTreeNode,
        OcrMetricsTreePayload,
        DocumentSummary,
        DocumentDetail,
        DocumentsPayload,
        DocumentRegionsPayload,
        OverlayBox,
        TextRegionSpan,
        PageTextRecord,
        DocumentTextPayload,
        FolderDialogResponse,
        PreviewImagesPayload,
        LogRecord,
        LogsPayload,
        RealtimeEventRecord,
        OcrReplayPayload,
        DiagnosticRunRecord,
        DiagnosticRunsPayload,
        DiagnosticSpanRecord,
        DiagnosticEventRecord,
        DiagnosticTraceSummary,
        DiagnosticTracePayload,
        DiagnosticWorkUnitRecord,
        DiagnosticModelLeaseRecord,
        DiagnosticProgressSummary,
        DiagnosticProgressPayload,
        DiagnosticBreakdownRecord,
        DiagnosticSlowSpanRecord,
        DiagnosticRecommendationRecord,
        DiagnosticAnalyticsSummary,
        DiagnosticAnalyticsPayload,
        DiagnosticModelsPayload
    )),
    tags(
        (name = "system"),
        (name = "ingest"),
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

struct BinaryImageResponse;

impl Modify for BinaryImageResponse {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for (path_key, content_types) in [
            (
                "/api/documents/{file_hash}/preview-images/{variant}/{page_no}",
                &["image/png", "image/jpeg"][..],
            ),
            (
                "/api/documents/{file_hash}/regions/{region_id}/snippet",
                &["image/png"][..],
            ),
        ] {
            let Some(path) = openapi.paths.paths.get_mut(path_key) else {
                continue;
            };
            let Some(operation) = path.get.as_mut() else {
                continue;
            };
            let Some(utoipa::openapi::RefOr::T(response)) =
                operation.responses.responses.get_mut("200")
            else {
                continue;
            };
            for content_type in content_types {
                if let Some(content) = response.content.get_mut(*content_type) {
                    content.schema = Some(binary_schema());
                }
            }
        }
    }
}

fn binary_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{KnownFormat, ObjectBuilder, Schema, SchemaFormat, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(Type::String)
            .format(Some(SchemaFormat::KnownFormat(KnownFormat::Binary)))
            .build(),
    ))
}

#[utoipa::path(get, path = "/api/health", tag = "system", responses((status = 200, body = HealthPayload)))]
const fn health_doc() {}

#[utoipa::path(get, path = "/api/status", tag = "system", responses((status = 200, body = StatusPayload)))]
const fn status_doc() {}

#[utoipa::path(get, path = "/api/openapi.json", tag = "system", responses((status = 200, description = "OpenAPI JSON")))]
const fn openapi_doc() {}

#[utoipa::path(post, path = "/api/system/folder-dialog", tag = "system", responses((status = 200, body = FolderDialogResponse)))]
const fn folder_dialog_doc() {}

#[utoipa::path(post, path = "/api/ingest/start", tag = "ingest", request_body = IngestStartRequest, responses((status = 202, description = "Ingest run queued", body = IngestStartResponse), (status = 400, body = ErrorPayload)))]
const fn start_ingest_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs", tag = "ingest", responses((status = 200, body = IngestRunsPayload)))]
const fn list_runs_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
const fn get_run_doc() {}

#[utoipa::path(post, path = "/api/ingest/runs/{run_id}/stop", tag = "ingest", params(("run_id" = String, Path)), responses((status = 202, description = "Stop requested", body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
const fn stop_run_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/events", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, description = "Server-sent run snapshots", body = String, content_type = "text/event-stream"), (status = 404, body = ErrorPayload)))]
const fn run_events_doc() {}

#[utoipa::path(get, path = "/api/ocr/events", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("file_hash" = Option<String>, Query), ("page_no" = Option<u32>, Query), ("since_sequence" = Option<u64>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = OcrReplayPayload)))]
const fn ocr_events_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/runs", tag = "diagnostics", params(("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticRunsPayload)))]
const fn diagnostics_runs_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/trace", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("file_hash" = Option<String>, Query), ("page_no" = Option<u32>, Query), ("status" = Option<String>, Query), ("q" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticTracePayload)))]
const fn diagnostics_trace_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/progress", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticProgressPayload)))]
const fn diagnostics_progress_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/analytics", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticAnalyticsPayload)))]
const fn diagnostics_analytics_doc() {}

#[utoipa::path(get, path = "/api/diagnostics/models", tag = "diagnostics", params(("run_id" = Option<String>, Query), ("limit" = Option<u32>, Query)), responses((status = 200, body = DiagnosticModelsPayload)))]
const fn diagnostics_models_doc() {}

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

#[utoipa::path(get, path = "/api/documents/{file_hash}/regions", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentRegionsPayload)))]
const fn document_regions_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/regions/{region_id}/snippet", tag = "documents", params(("file_hash" = String, Path), ("region_id" = String, Path)), responses((status = 200, description = "Region image snippet bytes", content((String = "image/png"))), (status = 404, body = ErrorPayload)))]
const fn region_snippet_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/text", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentTextPayload)))]
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

#[utoipa::path(get, path = "/api/logs/recent", tag = "logs", params(("limit" = Option<u32>, Query)), responses((status = 200, body = LogsPayload)))]
const fn logs_doc() {}
