//! Integration tests for the Trapo HTTP API and persisted schema contract.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use duckdb::params;
use tower::ServiceExt;
use trapo_server::{ApiDoc, AppState, Repository, ServerConfig, build_router};
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
async fn shutdown_route_requires_confirmation_and_blocks_new_work() -> anyhow::Result<()> {
    let state = test_state().await?;
    let app = build_router(state.clone());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/system/shutdown")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"confirm":"shutdown"}"#))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/system/shutdown")
                .header("content-type", "application/json")
                .header("x-trapo-intent", "shutdown")
                .body(Body::from(r#"{"confirm":"shutdown"}"#))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(payload["state"], "shutting_down");
    assert_eq!(payload["source"], "api");

    let scan_root = state.config().app_root.join("scan-after-shutdown");
    std::fs::create_dir_all(&scan_root)?;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ingest/start")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "root_path": scan_root.to_string_lossy().to_string() })
                        .to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    state.complete_shutdown().await;
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
                .uri("/api/ingest/runs/missing-run/resume")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

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
    let preview_images_path = format!(
        "/api/documents/{{{}}}/preview-images/{{{}}}/{{{}}}",
        "file_hash", "variant", "page_no"
    );
    let paths = value["paths"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("OpenAPI paths were not an object"))?;
    for path in [
        "/api/health",
        "/api/status",
        "/api/search",
        "/api/system/shutdown",
        "/api/ingest/start",
        "/api/ingest/runs/{run_id}/events",
        "/api/ingest/runs/{run_id}/resume",
        "/api/models/{model_id}/events",
        "/api/documents/{file_hash}/text",
        "/api/documents/{file_hash}/regions",
        "/api/documents/{file_hash}/regions/{region_id}/snippet",
        preview_images_path.as_str(),
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
        value["paths"][preview_images_path.as_str()]["get"]["responses"]["200"]["content"]["image/png"]
            ["schema"]["format"],
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

#[test]
fn openapi_page_fields_remain_numeric() -> anyhow::Result<()> {
    let value = serde_json::to_value(ApiDoc::openapi())?;
    let preview_images_path = format!(
        "/api/documents/{{{}}}/preview-images/{{{}}}/{{{}}}",
        "file_hash", "variant", "page_no"
    );
    for (schema_name, field_name) in [
        ("DocumentSummary", "page_count"),
        ("DocumentSummary", "current_page"),
        ("DocumentDetail", "page_count"),
        ("DocumentDetail", "current_page"),
        ("IngestRunRecord", "current_page"),
        ("OverlayBox", "page_no"),
        ("PageTextRecord", "page_no"),
        ("TextRegionSpan", "page_no"),
    ] {
        let schema = &value["components"]["schemas"][schema_name]["properties"][field_name];
        assert_integer_schema(schema, &format!("{schema_name}.{field_name}"));
    }

    let parameters = value["paths"][preview_images_path.as_str()]["get"]["parameters"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("preview route parameters were not an array"))?;
    let page_no = parameters
        .iter()
        .find(|parameter| parameter["name"].as_str() == Some("page_no"))
        .ok_or_else(|| anyhow::anyhow!("preview route is missing page_no parameter"))?;
    assert_integer_schema(
        &page_no["schema"],
        &format!("GET {preview_images_path} page_no"),
    );
    Ok(())
}

#[tokio::test]
async fn duckdb_page_columns_remain_integer() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let database_path = temp.path().join("trapo.duckdb");
    let repository = Repository::open(database_path.clone()).await?;
    drop(repository);
    let conn = duckdb::Connection::open(&database_path)?;

    for (table, column) in [
        ("files", "page_count"),
        ("ingest_work_units", "page_no"),
        ("document_pages", "page_no"),
        ("document_preview_images", "page_no"),
        ("document_page_ocr", "page_no"),
        ("document_run_page_ocr", "page_no"),
        ("document_regions", "page_no"),
        ("document_text_region_links", "page_no"),
        ("document_terms", "page_no"),
        ("annotation_visibility_overrides", "page_no"),
        ("document_region_annotations", "page_no"),
        ("ocr_page_metrics", "page_no"),
        ("ocr_stream_events", "page_no"),
        ("ingest_diagnostic_spans", "page_no"),
        ("ingest_diagnostic_events", "page_no"),
    ] {
        assert_eq!(
            column_type(&conn, table, column)?,
            "INTEGER",
            "{table}.{column} must remain INTEGER"
        );
    }
    Ok(())
}

fn assert_integer_schema(schema: &serde_json::Value, name: &str) {
    assert!(
        schema_type_contains(schema, "integer"),
        "{name} must include integer type: {schema}"
    );
    assert!(
        !schema_type_contains(schema, "string"),
        "{name} must not include string type: {schema}"
    );
}

fn schema_type_contains(schema: &serde_json::Value, expected: &str) -> bool {
    match &schema["type"] {
        serde_json::Value::String(value) => value == expected,
        serde_json::Value::Array(values) => {
            values.iter().any(|value| value.as_str() == Some(expected))
        }
        _ => false,
    }
}

fn column_type(conn: &duckdb::Connection, table: &str, column: &str) -> anyhow::Result<String> {
    Ok(conn.query_row(
        "SELECT data_type FROM information_schema.columns WHERE table_name = ? AND column_name = ?",
        params![table, column],
        |row| row.get::<_, String>(0),
    )?)
}

async fn test_state() -> anyhow::Result<AppState> {
    let temp = tempfile::tempdir()?;
    let root = temp.keep();
    let client_dist = root.join("src").join("trapo-client").join("dist");
    std::fs::create_dir_all(&client_dist)?;
    std::fs::write(client_dist.join("index.html"), "<!doctype html>")?;
    install_placeholder_cpu_runtime(&root)?;
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

fn install_placeholder_cpu_runtime(root: &std::path::Path) -> anyhow::Result<()> {
    for (platform, library_name) in [
        ("windows-x86_64-cpu", "uocr-ffi.dll"),
        ("windows-arm64-cpu", "uocr-ffi.dll"),
        ("linux-x86_64-cpu", "libuocr-ffi.so"),
        ("linux-arm64-cpu", "libuocr-ffi.so"),
        ("macos-arm64-cpu", "libuocr-ffi.dylib"),
    ] {
        let runtime_dir = root
            .join("thirdparty")
            .join("uocr-runtime")
            .join(platform)
            .join("bin");
        std::fs::create_dir_all(&runtime_dir)?;
        std::fs::write(runtime_dir.join(library_name), &[] as &[u8])?;
    }
    Ok(())
}
