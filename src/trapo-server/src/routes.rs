use axum::{
    Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};
use utoipa_scalar::{Scalar, Servable};

use crate::{
    app::AppState,
    error::{AppError, Result},
    logger::LogFilter,
    openapi::openapi_document,
    realtime,
    types::SettingsUpdateRequest,
    workbench_types::{
        GenerateEmbeddingRequest, HybridSearchRequest, IngestStartRequest, TextIndexRequest,
    },
};

#[derive(Debug, Deserialize)]
struct LimitQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct LogsQuery {
    limit: Option<usize>,
    level: Option<String>,
    component: Option<String>,
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RunScopeQuery {
    run_id: Option<String>,
    run: Option<String>,
    run_engine_id: Option<String>,
    result: Option<String>,
}

impl RunScopeQuery {
    fn run_id(&self) -> Option<&str> {
        self.run_id
            .as_deref()
            .or(self.run.as_deref())
            .filter(|value| !value.is_empty())
    }

    fn run_engine_id(&self) -> Option<&str> {
        self.run_engine_id
            .as_deref()
            .or(self.result.as_deref())
            .filter(|value| !value.is_empty())
    }
}

#[derive(Debug, Deserialize)]
struct PreviewResultsQuery {
    file_hash: String,
}

pub(crate) fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/status", get(status))
        .route("/api/openapi.json", get(openapi_json))
        .route("/api/system/folder-dialog", post(folder_dialog))
        .route("/api/system/shutdown", post(shutdown))
        .route("/api/ingest/engines", get(ingest_engines))
        .route("/api/ingest/start", post(start_ingest))
        .route("/api/ingest/runs", get(list_runs))
        .route("/api/ingest/metrics/recent", get(recent_metrics))
        .route("/api/ingest/runs/{run_id}", get(get_run))
        .route(
            "/api/ingest/runs/{run_id}/preview-results",
            get(preview_results),
        )
        .route("/api/ingest/runs/{run_id}/metrics", get(run_metrics))
        .route("/api/ingest/runs/{run_id}/resume", post(resume_run))
        .route("/api/ingest/runs/{run_id}/stop", post(stop_run))
        .route("/api/ingest/runs/{run_id}/events", get(run_events))
        .route("/api/ocr/events", get(ocr_events))
        .route("/api/diagnostics/runs", get(diagnostics_runs))
        .route("/api/diagnostics/trace", get(diagnostics_trace))
        .route("/api/diagnostics/waterfall", get(diagnostics_waterfall))
        .route("/api/diagnostics/progress", get(diagnostics_progress))
        .route(
            "/api/diagnostics/work-units/{work_unit_id}",
            get(diagnostics_work_unit_detail),
        )
        .route("/api/diagnostics/analytics", get(diagnostics_analytics))
        .route("/api/diagnostics/models", get(diagnostics_models))
        .route("/api/rag/text-index", post(start_text_index))
        .route("/api/rag/embeddings", post(generate_embedding))
        .route("/api/rag/embedding-models/used", get(used_embedding_models))
        .route("/api/rag/search", post(hybrid_search))
        .route("/api/documents", get(list_documents))
        .route("/api/search", get(list_documents))
        .route("/api/documents/{file_hash}", get(get_document))
        .route("/api/documents/{file_hash}/regions", get(document_regions))
        .route(
            "/api/documents/{file_hash}/regions/{region_id}/snippet",
            get(region_snippet),
        )
        .route("/api/documents/{file_hash}/text", get(document_text))
        .route(
            "/api/documents/{file_hash}/preview-images",
            get(preview_images),
        )
        .route(
            "/api/documents/{file_hash}/preview-images/{variant}/{page_no}",
            get(preview_image),
        )
        .route("/api/settings", get(settings).put(update_settings))
        .route("/api/models", get(models))
        .route("/api/models/{model_id}/download", post(download_model))
        .route("/api/models/{model_id}/select", post(select_model))
        .route("/api/models/{model_id}/cancel", post(cancel_model))
        .route("/api/models/{model_id}/events", get(model_events))
        .route("/api/logs/recent", get(logs))
        .route("/api/logs/export", get(export_logs))
        .route("/api/events", get(websocket))
        .merge(Scalar::with_url("/scalar", openapi_document()));

    Router::new()
        .merge(api)
        .fallback_service(spa_service(&state.config().client_dist))
        .with_state(state)
}

fn spa_service(client_dist: &std::path::Path) -> ServeDir<ServeFile> {
    ServeDir::new(client_dist).fallback(ServeFile::new(client_dist.join("index.html")))
}

fn limit_query_u32(value: usize, max: u32) -> u32 {
    u32::try_from(value).map_or(max, |limit| limit.min(max))
}

async fn health(State(_state): State<AppState>) -> Json<crate::types::HealthPayload> {
    Json(AppState::health())
}

async fn status(State(state): State<AppState>) -> Json<crate::types::StatusPayload> {
    Json(state.status().await)
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi_document())
}

async fn folder_dialog(
    State(state): State<AppState>,
) -> Json<crate::workbench_types::FolderDialogResponse> {
    Json(state.folder_dialog().await)
}

async fn start_ingest(
    State(state): State<AppState>,
    Json(request): Json<IngestStartRequest>,
) -> Result<(
    StatusCode,
    Json<crate::workbench_types::IngestStartResponse>,
)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.start_ingest(request).await?),
    ))
}

async fn ingest_engines(
    State(state): State<AppState>,
) -> Json<crate::workbench_types::IngestEnginesPayload> {
    Json(state.ingest_engines().await)
}

async fn list_runs(
    State(state): State<AppState>,
) -> Json<crate::workbench_types::IngestRunsPayload> {
    Json(state.list_runs().await)
}

async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<crate::workbench_types::IngestRunRecord>> {
    Ok(Json(state.get_run(&run_id).await?))
}

async fn preview_results(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<PreviewResultsQuery>,
) -> Result<Json<crate::workbench_types::IngestPreviewResultsPayload>> {
    Ok(Json(
        state.preview_results(&run_id, &query.file_hash).await?,
    ))
}

async fn stop_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<(StatusCode, Json<crate::workbench_types::IngestRunRecord>)> {
    Ok((StatusCode::ACCEPTED, Json(state.stop_run(&run_id).await?)))
}

async fn resume_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<(
    StatusCode,
    Json<crate::workbench_types::IngestStartResponse>,
)> {
    Ok((StatusCode::ACCEPTED, Json(state.resume_run(&run_id).await?)))
}

async fn run_events(State(state): State<AppState>, Path(run_id): Path<String>) -> Result<Response> {
    let run = state.get_run(&run_id).await?;
    let data = serde_json::to_string(&run)?;
    Ok((
        [(header::CONTENT_TYPE, "text/event-stream")],
        format!("event: snapshot\ndata: {data}\n\n"),
    )
        .into_response())
}

async fn recent_metrics(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<crate::workbench_types::OcrMetricsTreePayload>> {
    Ok(Json(
        state
            .run_metrics(None, limit_query_u32(query.limit.unwrap_or(50), 100_000))
            .await?,
    ))
}

async fn run_metrics(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<crate::workbench_types::OcrMetricsTreePayload>> {
    Ok(Json(state.run_metrics(Some(&run_id), 100_000).await?))
}

async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<crate::workbench_types::DocumentsPayload>> {
    Ok(Json(state.list_documents(query.q).await?))
}

async fn start_text_index(
    State(state): State<AppState>,
    Json(request): Json<TextIndexRequest>,
) -> Result<(StatusCode, Json<crate::workbench_types::TextIndexResponse>)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.start_text_index(request).await?),
    ))
}

async fn generate_embedding(
    State(state): State<AppState>,
    Json(request): Json<GenerateEmbeddingRequest>,
) -> Result<(
    StatusCode,
    Json<crate::workbench_types::GenerateEmbeddingResponse>,
)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.start_generate_embedding(request).await?),
    ))
}

async fn used_embedding_models(
    State(state): State<AppState>,
) -> Result<Json<crate::workbench_types::UsedEmbeddingModelsPayload>> {
    Ok(Json(state.used_embedding_models().await?))
}

async fn hybrid_search(
    State(state): State<AppState>,
    Json(request): Json<HybridSearchRequest>,
) -> Result<Json<crate::workbench_types::HybridSearchResponse>> {
    Ok(Json(state.hybrid_search(request).await?))
}

async fn get_document(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Result<Json<crate::workbench_types::DocumentDetail>> {
    Ok(Json(state.get_document(&file_hash).await?))
}

async fn document_regions(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
    Query(query): Query<RunScopeQuery>,
) -> Result<Json<crate::workbench_types::DocumentRegionsPayload>> {
    Ok(Json(
        state
            .document_regions(&file_hash, query.run_id(), query.run_engine_id())
            .await?,
    ))
}

async fn region_snippet(
    State(state): State<AppState>,
    Path((file_hash, region_id)): Path<(String, String)>,
) -> Result<Response> {
    let Some(path) = state
        .region_snippet_path_for_request(&file_hash, &region_id)
        .await
    else {
        return Err(AppError::NotFound("region snippet not found".to_string()));
    };
    let bytes = tokio::fs::read(&path).await?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes).into_response())
}

async fn document_text(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
    Query(query): Query<RunScopeQuery>,
) -> Result<Json<crate::workbench_types::DocumentTextPayload>> {
    Ok(Json(
        state
            .document_text(&file_hash, query.run_id(), query.run_engine_id())
            .await?,
    ))
}

async fn preview_images(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Json<crate::workbench_types::PreviewImagesPayload> {
    Json(state.preview_images(&file_hash).await)
}

async fn preview_image(
    State(state): State<AppState>,
    Path((file_hash, variant, page_no)): Path<(String, String, u32)>,
) -> Result<Response> {
    let Some(path) = state
        .preview_image_path(&file_hash, &variant, page_no)
        .await
    else {
        return Err(AppError::NotFound("preview image not found".to_string()));
    };
    let bytes = tokio::fs::read(&path).await?;
    let content_type = mime_guess::from_path(&path).first_or_octet_stream();
    Ok(([(header::CONTENT_TYPE, content_type.as_ref())], bytes).into_response())
}

async fn settings(State(state): State<AppState>) -> Json<crate::types::SettingsPayload> {
    Json(state.settings().await)
}

async fn update_settings(
    State(state): State<AppState>,
    Json(request): Json<SettingsUpdateRequest>,
) -> Result<Json<crate::types::SettingsPayload>> {
    Ok(Json(state.update_settings(request).await?))
}

async fn logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Json<crate::workbench_types::LogsPayload> {
    Json(state.logs(
        query.limit.unwrap_or(200),
        &LogFilter {
            level: non_empty_query_value(query.level),
            component: non_empty_query_value(query.component),
            query: non_empty_query_value(query.q),
        },
    ))
}

async fn export_logs(State(state): State<AppState>) -> Response {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        state.export_logs_plain(),
    )
        .into_response()
}

async fn websocket(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let hub = state.hub();
    ws.on_upgrade(move |socket| realtime::websocket(socket, hub))
}

fn non_empty_query_value(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

include!("routes_diagnostics.rs");
include!("routes_models.rs");
include!("routes_system.rs");
