use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use trapo_server::{AppState, ServerConfig, build_router, openapi::ApiDoc};
use utoipa::OpenApi;

#[tokio::test]
async fn health_and_openapi_are_served() -> anyhow::Result<()> {
    let state = test_state().await?;
    let app = build_router(state);

    let response = app
        .clone()
        .oneshot(Request::builder().uri("/api/health").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let value: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(value["service"], "trapo-server");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/openapi.json")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn invalid_profile_update_is_rejected() -> anyhow::Result<()> {
    let app = build_router(test_state().await?);
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"default_profile":"missing"}"#))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn parity_mutation_routes_return_accepted() -> anyhow::Result<()> {
    let state = test_state().await?;
    let app = build_router(state.clone());
    let scan_root = state.config().app_root.join("scan-root");
    std::fs::create_dir_all(&scan_root)?;
    let request = serde_json::json!({ "root_path": scan_root.to_string_lossy().to_string() });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ingest/start")
                .header("content-type", "application/json")
                .body(Body::from(request.to_string()))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let start: serde_json::Value = serde_json::from_slice(&body)?;
    let run = &start["run"];
    let run_id = run["run_id"].as_str().unwrap_or_default();
    assert!(!run_id.is_empty());
    assert!(start["documents"].as_array().is_some());
    assert!(start["replay_since_sequence"].as_u64().is_some());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/ingest/runs/{run_id}/stop"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/models/unlimited-ocr-q4-k-m/select")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/models/unlimited-ocr-q4-k-m/cancel")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    Ok(())
}

#[test]
fn openapi_serves_trapo_workbench_contract() -> anyhow::Result<()> {
    let value = serde_json::to_value(ApiDoc::openapi())?;
    let paths = value["paths"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("OpenAPI paths were not an object"))?;
    for path in [
        "/api/health",
        "/api/status",
        "/api/search",
        "/api/ingest/start",
        "/api/ingest/runs/{run_id}/events",
        "/api/models/{model_id}/events",
        "/api/documents/{file_hash}/text",
        "/api/documents/{file_hash}/regions",
        "/api/documents/{file_hash}/regions/{region_id}/snippet",
        "/api/documents/{file_hash}/preview-images/{variant}/{page_no}",
    ] {
        assert!(paths.contains_key(path), "missing OpenAPI path {path}");
    }
    assert!(value["components"]["schemas"]["ModelDownloadEvent"].is_object());
    assert_eq!(
        value["paths"]["/api/ingest/start"]["post"]["responses"]["202"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/IngestStartResponse"
    );
    assert_eq!(
        value["paths"]["/api/documents/{file_hash}/text"]["get"]["responses"]["200"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/DocumentTextPayload"
    );
    assert_eq!(
        value["paths"]["/api/documents/{file_hash}/preview-images/{variant}/{page_no}"]["get"]["responses"]
            ["200"]["content"]["image/png"]["schema"]["format"],
        "binary"
    );
    assert_eq!(
        value["paths"]["/api/documents/{file_hash}/regions/{region_id}/snippet"]["get"]["responses"]
            ["200"]["content"]["image/png"]["schema"]["format"],
        "binary"
    );
    assert!(
        value["paths"]["/api/models/{model_id}/select"]["post"]["responses"]["202"].is_object()
    );
    assert!(
        value["paths"]["/api/models/{model_id}/cancel"]["post"]["responses"]["202"].is_object()
    );
    Ok(())
}

async fn test_state() -> anyhow::Result<AppState> {
    let temp = tempfile::tempdir()?;
    let root = temp.keep();
    let client_dist = root.join("src").join("trapo-client").join("dist");
    std::fs::create_dir_all(&client_dist)?;
    std::fs::write(client_dist.join("index.html"), "<!doctype html>")?;
    Ok(AppState::new(ServerConfig {
        app_root: root.clone(),
        client_dist,
        data_dir: root.join("data"),
        cache_dir: root.join("cache"),
        log_dir: root.join("logs"),
        model_dir: root.join("models"),
        database_path: root.join("data").join("trapo.duckdb"),
        pdfium_library_dir: None,
        host: "127.0.0.1".to_string(),
        port: 0,
        open_browser: false,
    })
    .await?)
}
