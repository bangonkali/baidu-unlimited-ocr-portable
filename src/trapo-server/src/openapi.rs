#![allow(dead_code)]

use utoipa::{Modify, OpenApi};

use crate::{error::ErrorPayload, types::*, workbench_types::*};

#[derive(OpenApi)]
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
        run_metrics_doc,
        recent_metrics_doc,
        list_documents_doc,
        search_documents_doc,
        get_document_doc,
        document_regions_doc,
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
        LogsPayload
    )),
    tags(
        (name = "system"),
        (name = "ingest"),
        (name = "documents"),
        (name = "settings"),
        (name = "models"),
        (name = "logs")
    ),
    modifiers(&BinaryImageResponse)
)]
pub struct ApiDoc;

struct BinaryImageResponse;

impl Modify for BinaryImageResponse {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let Some(path) = openapi
            .paths
            .paths
            .get_mut("/api/documents/{file_hash}/preview-images/{variant}/{page_no}")
        else {
            return;
        };
        let Some(operation) = path.get.as_mut() else {
            return;
        };
        let Some(utoipa::openapi::RefOr::T(response)) =
            operation.responses.responses.get_mut("200")
        else {
            return;
        };
        for content_type in ["image/png", "image/jpeg"] {
            if let Some(content) = response.content.get_mut(content_type) {
                content.schema = Some(binary_schema());
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
fn health_doc() {}

#[utoipa::path(get, path = "/api/status", tag = "system", responses((status = 200, body = StatusPayload)))]
fn status_doc() {}

#[utoipa::path(get, path = "/api/openapi.json", tag = "system", responses((status = 200, description = "OpenAPI JSON")))]
fn openapi_doc() {}

#[utoipa::path(post, path = "/api/system/folder-dialog", tag = "system", responses((status = 200, body = FolderDialogResponse)))]
fn folder_dialog_doc() {}

#[utoipa::path(post, path = "/api/ingest/start", tag = "ingest", request_body = IngestStartRequest, responses((status = 202, description = "Ingest run queued", body = IngestRunRecord), (status = 400, body = ErrorPayload)))]
fn start_ingest_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs", tag = "ingest", responses((status = 200, body = IngestRunsPayload)))]
fn list_runs_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
fn get_run_doc() {}

#[utoipa::path(post, path = "/api/ingest/runs/{run_id}/stop", tag = "ingest", params(("run_id" = String, Path)), responses((status = 202, description = "Stop requested", body = IngestRunRecord), (status = 404, body = ErrorPayload)))]
fn stop_run_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/events", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, description = "Server-sent run snapshots", body = String, content_type = "text/event-stream"), (status = 404, body = ErrorPayload)))]
fn run_events_doc() {}

#[utoipa::path(get, path = "/api/ingest/runs/{run_id}/metrics", tag = "ingest", params(("run_id" = String, Path)), responses((status = 200, body = OcrMetricsTreePayload)))]
fn run_metrics_doc() {}

#[utoipa::path(get, path = "/api/ingest/metrics/recent", tag = "ingest", params(("limit" = Option<u32>, Query)), responses((status = 200, body = OcrMetricsTreePayload)))]
fn recent_metrics_doc() {}

#[utoipa::path(get, path = "/api/documents", tag = "documents", params(("q" = Option<String>, Query)), responses((status = 200, body = DocumentsPayload)))]
fn list_documents_doc() {}

#[utoipa::path(get, path = "/api/search", tag = "documents", params(("q" = Option<String>, Query)), responses((status = 200, description = "DuckDB-backed document text search", body = DocumentsPayload)))]
fn search_documents_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentDetail), (status = 404, body = ErrorPayload)))]
fn get_document_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/regions", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentRegionsPayload)))]
fn document_regions_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/text", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = DocumentTextPayload)))]
fn document_text_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/preview-images", tag = "documents", params(("file_hash" = String, Path)), responses((status = 200, body = PreviewImagesPayload)))]
fn preview_images_doc() {}

#[utoipa::path(get, path = "/api/documents/{file_hash}/preview-images/{variant}/{page_no}", tag = "documents", params(("file_hash" = String, Path), ("variant" = String, Path), ("page_no" = u32, Path)), responses((status = 200, description = "Preview image bytes", content((String = "image/png"), (String = "image/jpeg"))), (status = 404, body = ErrorPayload)))]
fn preview_image_doc() {}

#[utoipa::path(get, path = "/api/settings", tag = "settings", responses((status = 200, body = SettingsPayload)))]
fn settings_doc() {}

#[utoipa::path(put, path = "/api/settings", tag = "settings", request_body = SettingsUpdateRequest, responses((status = 200, body = SettingsPayload), (status = 400, body = ErrorPayload)))]
fn update_settings_doc() {}

#[utoipa::path(get, path = "/api/models", tag = "models", responses((status = 200, body = ModelsPayload)))]
fn models_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/download", tag = "models", params(("model_id" = String, Path)), request_body = ModelDownloadRequest, responses((status = 202, body = ModelDownloadRecord), (status = 409, body = ErrorPayload)))]
fn download_model_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/select", tag = "models", params(("model_id" = String, Path)), responses((status = 202, description = "Model selected for subsequent ingest runs", body = ModelSelectRecord), (status = 400, body = ErrorPayload)))]
fn select_model_doc() {}

#[utoipa::path(post, path = "/api/models/{model_id}/cancel", tag = "models", params(("model_id" = String, Path)), responses((status = 202, description = "Download cancellation requested", body = ModelDownloadRecord), (status = 400, body = ErrorPayload)))]
fn cancel_model_doc() {}

#[utoipa::path(get, path = "/api/models/{model_id}/events", tag = "models", params(("model_id" = String, Path)), responses((status = 200, description = "Server-sent model download progress events", body = String, content_type = "text/event-stream"), (status = 400, body = ErrorPayload)))]
fn model_events_doc() {}

#[utoipa::path(get, path = "/api/logs/recent", tag = "logs", params(("limit" = Option<u32>, Query)), responses((status = 200, body = LogsPayload)))]
fn logs_doc() {}
