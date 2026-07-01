use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use trapo_server::{AppState, ServerConfig, build_router};

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
        log_dir: root.join(".logs"),
        model_dir: root.join("models"),
        database_path: root.join("data").join("trapo.duckdb"),
        pdfium_library_dir: None,
        host: "127.0.0.1".to_string(),
        port: 0,
        open_browser: false,
    })
    .await?)
}
