use axum::{
    Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    app::AppState,
    error::{AppError, Result},
    openapi::ApiDoc,
    realtime,
    types::{ModelDownloadRequest, SettingsUpdateRequest},
    workbench_types::IngestStartRequest,
};

#[derive(Debug, Deserialize)]
struct LimitQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/status", get(status))
        .route("/api/openapi.json", get(openapi_json))
        .route("/api/system/folder-dialog", post(folder_dialog))
        .route("/api/ingest/start", post(start_ingest))
        .route("/api/ingest/runs", get(list_runs))
        .route("/api/ingest/metrics/recent", get(recent_metrics))
        .route("/api/ingest/runs/{run_id}", get(get_run))
        .route("/api/ingest/runs/{run_id}/metrics", get(run_metrics))
        .route("/api/ingest/runs/{run_id}/stop", post(stop_run))
        .route("/api/ingest/runs/{run_id}/events", get(run_events))
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
        .route("/api/events", get(websocket))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()));

    Router::new()
        .merge(api)
        .fallback_service(spa_service(state.config().client_dist.clone()))
        .with_state(state)
}

fn spa_service(client_dist: std::path::PathBuf) -> ServeDir<ServeFile> {
    ServeDir::new(&client_dist).fallback(ServeFile::new(client_dist.join("index.html")))
}

async fn health(State(state): State<AppState>) -> Json<crate::types::HealthPayload> {
    Json(state.health().await)
}

async fn status(State(state): State<AppState>) -> Json<crate::types::StatusPayload> {
    Json(state.status().await)
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn folder_dialog(
    State(state): State<AppState>,
) -> Json<crate::workbench_types::FolderDialogResponse> {
    Json(state.folder_dialog().await)
}

async fn start_ingest(
    State(state): State<AppState>,
    Json(request): Json<IngestStartRequest>,
) -> Result<(StatusCode, Json<crate::workbench_types::IngestRunRecord>)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.start_ingest(request).await?),
    ))
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

async fn stop_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<(StatusCode, Json<crate::workbench_types::IngestRunRecord>)> {
    Ok((StatusCode::ACCEPTED, Json(state.stop_run(&run_id).await?)))
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
            .run_metrics(None, query.limit.unwrap_or(50) as u32)
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

async fn get_document(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Result<Json<crate::workbench_types::DocumentDetail>> {
    Ok(Json(state.get_document(&file_hash).await?))
}

async fn document_regions(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> Json<crate::workbench_types::DocumentRegionsPayload> {
    Json(state.document_regions(&file_hash).await)
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
) -> Json<crate::workbench_types::DocumentTextPayload> {
    Json(state.document_text(&file_hash).await)
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

async fn models(State(state): State<AppState>) -> Json<crate::types::ModelsPayload> {
    Json(state.models().await)
}

async fn download_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Json(request): Json<ModelDownloadRequest>,
) -> Result<(StatusCode, Json<crate::types::ModelDownloadRecord>)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.start_model_download(&model_id, request).await?),
    ))
}

async fn select_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<(StatusCode, Json<crate::types::ModelSelectRecord>)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.select_model(&model_id).await?),
    ))
}

async fn cancel_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<(StatusCode, Json<crate::types::ModelDownloadRecord>)> {
    Ok((
        StatusCode::ACCEPTED,
        Json(state.cancel_model_download(&model_id).await?),
    ))
}

async fn model_events(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Response> {
    let record = state.model_download_event(&model_id).await?;
    let data = serde_json::to_string(&record)?;
    Ok((
        [(header::CONTENT_TYPE, "text/event-stream")],
        format!("event: model\ndata: {data}\n\n"),
    )
        .into_response())
}

async fn logs(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Json<crate::workbench_types::LogsPayload> {
    Json(state.logs(query.limit.unwrap_or(200)).await)
}

async fn websocket(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    let hub = state.hub();
    ws.on_upgrade(move |socket| realtime::websocket(socket, hub))
}
