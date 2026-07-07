//! Integration tests for the Trapo HTTP API and persisted schema contract.

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use duckdb::params;
use tower::ServiceExt;
use trapo_server::{ApiDoc, AppState, Repository, ServerConfig, build_router};
use utoipa::OpenApi;
use uuid::Uuid;

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

#[tokio::test]
async fn ingest_engines_route_returns_default_presets() -> anyhow::Result<()> {
    let app = build_router(test_state().await?);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ingest/engines")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    let engines = payload["engines"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("engines missing: {payload}"))?;
    assert_eq!(engines.len(), 6);
    assert!(
        engines
            .iter()
            .any(|engine| engine["engine_id"] == "unlimited-ocr-ffi"
                && engine["available"] == true)
    );
    assert!(
        engines
            .iter()
            .any(|engine| engine["engine_id"] == "dots-mocr-gguf"
                && engine["availability"] == "missing_model")
    );
    Ok(())
}

#[tokio::test]
async fn text_index_route_chunks_long_cjk_pages_for_embedding_safety() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.keep();
    let config = test_config(&root)?;
    let repository = Repository::open(config.database_path.clone()).await?;
    drop(repository);
    let run_id = Uuid::now_v7().to_string();
    seed_completed_text_run(
        &config,
        &run_id,
        "file-long-cjk",
        &"饮用水卫生标准".repeat(180),
    )?;
    let state = AppState::new(config).await?;
    let app = build_router(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rag/text-index")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "source_run_id": run_id }).to_string(),
                ))?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    assert!(
        payload["segments_indexed"].as_u64().unwrap_or_default() > 1,
        "long CJK page should be chunked before FTS/embedding: {payload}"
    );
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
        "/api/ingest/engines",
        "/api/ingest/start",
        "/api/ingest/runs/{run_id}/events",
        "/api/ingest/runs/{run_id}/preview-results",
        "/api/ingest/runs/{run_id}/resume",
        "/api/diagnostics/waterfall",
        "/api/logs/export",
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

#[tokio::test]
async fn diagnostics_waterfall_route_returns_trace_rows() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.keep();
    let config = test_config(&root)?;
    let repository = Repository::open(config.database_path.clone()).await?;
    drop(repository);
    let run_id = Uuid::now_v7().to_string();
    let task_id = Uuid::now_v7().to_string();
    let text_task_id = Uuid::now_v7().to_string();
    let work_unit_id = Uuid::now_v7().to_string();
    let orphan_work_unit_id = Uuid::now_v7().to_string();
    let root_span_id = Uuid::now_v7().to_string();
    let page_span_id = Uuid::now_v7().to_string();
    let text_root_span_id = Uuid::now_v7().to_string();
    let text_page_span_id = Uuid::now_v7().to_string();
    let render_span_id = Uuid::now_v7().to_string();
    let ocr_work_unit_id = Uuid::now_v7().to_string();
    let ocr_span_id = Uuid::now_v7().to_string();
    let seed = DiagnosticWaterfallSeed {
        run: &run_id,
        task: &task_id,
        text_task: &text_task_id,
        work_unit: &work_unit_id,
        orphan_work_unit: &orphan_work_unit_id,
        root_span: &root_span_id,
        page_span: &page_span_id,
        text_root_span: &text_root_span_id,
        text_page_span: &text_page_span_id,
        render_span: &render_span_id,
        ocr_work_unit: &ocr_work_unit_id,
        ocr_span: &ocr_span_id,
    };
    seed_diagnostic_waterfall_run(&config, &seed)?;
    let app = build_router(AppState::new(config).await?);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/diagnostics/waterfall?run_id={run_id}"))
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    let rows = payload["rows"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("waterfall rows missing: {payload}"))?;
    assert_diagnostic_waterfall_hierarchy(rows, &seed, &payload)?;
    Ok(())
}

fn assert_diagnostic_waterfall_hierarchy(
    rows: &[serde_json::Value],
    seed: &DiagnosticWaterfallSeed<'_>,
    payload: &serde_json::Value,
) -> anyhow::Result<()> {
    assert!(
        rows.iter()
            .any(|row| row["row_id"] == format!("task:{}", seed.task))
    );
    let file_group_id = assert_ocr_waterfall_grouping(rows, seed, payload);
    assert_waterfall_task_folding(rows, seed, payload);
    assert_waterfall_work_unit_folding(rows, seed, payload);
    assert_rag_waterfall_grouping(rows, seed, payload)?;
    let ocr_page = rows
        .iter()
        .find(|row| row["span_id"] == seed.ocr_span)
        .ok_or_else(|| anyhow::anyhow!("ocr page span missing: {payload}"))?;
    assert_eq!(ocr_page["parent_row_id"], file_group_id);
    Ok(())
}

fn assert_ocr_waterfall_grouping(
    rows: &[serde_json::Value],
    seed: &DiagnosticWaterfallSeed<'_>,
    payload: &serde_json::Value,
) -> String {
    let run_root_id = format!("run:{}", seed.run);
    let ocr_group_id = format!("group:{}:ocr", seed.run);
    let file_group_id = format!("file-group:{}:file-waterfall:ocr", seed.run);
    assert!(
        rows.iter()
            .any(|row| row["row_id"] == run_root_id && row["row_source"] == "run")
    );
    assert!(
        rows.iter()
            .any(|row| { row["row_id"] == ocr_group_id && row["parent_row_id"] == run_root_id })
    );
    assert!(
        rows.iter()
            .any(|row| { row["row_id"] == file_group_id && row["parent_row_id"] == ocr_group_id })
    );
    let file_groups = rows
        .iter()
        .filter(|row| row["row_id"] == file_group_id)
        .collect::<Vec<_>>();
    assert_eq!(
        file_groups.len(),
        1,
        "OCR grouping must not create empty duplicate file groups: {payload}"
    );
    let file_group = file_groups[0];
    assert_eq!(file_group["label"], "trace.pdf");
    assert_eq!(file_group["filename"], "trace.pdf");
    assert!(
        file_group["child_count"].as_u64().unwrap_or_default() > 0,
        "file group should contain render and OCR spans: {payload}"
    );
    assert!(
        file_group["visual_duration_ms"]
            .as_f64()
            .unwrap_or_default()
            > 0.0,
        "file group should inherit visual bounds from children: {payload}"
    );
    file_group_id
}

fn assert_waterfall_task_folding(
    rows: &[serde_json::Value],
    seed: &DiagnosticWaterfallSeed<'_>,
    payload: &serde_json::Value,
) {
    assert!(
        rows.iter()
            .any(|row| row["pipeline_step"] == "generate_embedding")
    );
    assert!(
        !rows.iter().any(|row| row["span_id"] == seed.root_span),
        "duplicate embedding task diagnostic span should be folded into pipeline task: {payload}"
    );
    assert!(
        !rows.iter().any(|row| row["span_id"] == seed.text_root_span),
        "duplicate text-index task diagnostic span should be folded into pipeline task: {payload}"
    );
}

fn assert_waterfall_work_unit_folding(
    rows: &[serde_json::Value],
    seed: &DiagnosticWaterfallSeed<'_>,
    payload: &serde_json::Value,
) {
    assert!(
        !rows
            .iter()
            .any(|row| row["row_id"] == format!("work:{}", seed.work_unit)),
        "matched work unit should be folded into its diagnostic span: {payload}"
    );
    assert!(
        rows.iter().any(|row| {
            row["row_id"] == format!("work:{}", seed.orphan_work_unit)
                && row["row_source"] == "work_unit"
        }),
        "unmatched work units should remain visible: {payload}"
    );
}

fn assert_rag_waterfall_grouping(
    rows: &[serde_json::Value],
    seed: &DiagnosticWaterfallSeed<'_>,
    payload: &serde_json::Value,
) -> anyhow::Result<()> {
    let page = rows
        .iter()
        .find(|row| row["span_id"] == seed.page_span)
        .ok_or_else(|| anyhow::anyhow!("page span missing: {payload}"))?;
    let embedding_file_group_id =
        format!("file-group:{}:file-waterfall:generate_embedding", seed.task);
    assert_rag_file_group(
        rows,
        &embedding_file_group_id,
        &format!("task:{}", seed.task),
        payload,
    )?;
    assert_eq!(page["parent_row_id"], embedding_file_group_id);
    assert_eq!(page["task_id"], seed.task);
    assert_eq!(page["work_unit_id"], seed.work_unit);
    assert_eq!(page["row_source"], "diagnostic_span");
    assert_eq!(page["span_kind"], "embedding_page");
    assert_eq!(
        page["attributes"]["work_unit"]["work_unit_id"],
        seed.work_unit
    );
    let text_page = rows
        .iter()
        .find(|row| row["span_id"] == seed.text_page_span)
        .ok_or_else(|| anyhow::anyhow!("text page span missing: {payload}"))?;
    let text_file_group_id = format!("file-group:{}:file-waterfall:text_index", seed.text_task);
    assert_rag_file_group(
        rows,
        &text_file_group_id,
        &format!("task:{}", seed.text_task),
        payload,
    )?;
    assert_eq!(text_page["parent_row_id"], text_file_group_id);
    assert_eq!(text_page["filename"], "trace.pdf");
    assert!(page["visual_start_ms"].as_f64().is_some());
    Ok(())
}

fn assert_rag_file_group(
    rows: &[serde_json::Value],
    file_group_id: &str,
    task_row_id: &str,
    payload: &serde_json::Value,
) -> anyhow::Result<()> {
    let group = rows
        .iter()
        .find(|row| row["row_id"] == file_group_id)
        .ok_or_else(|| anyhow::anyhow!("missing RAG file group {file_group_id}: {payload}"))?;
    assert_eq!(group["parent_row_id"], task_row_id);
    assert_eq!(group["row_source"], "file_group");
    assert_eq!(group["label"], "trace.pdf");
    assert!(
        group["child_count"].as_u64().unwrap_or_default() > 0,
        "RAG file group should contain page spans: {payload}"
    );
    Ok(())
}

#[tokio::test]
async fn logs_export_route_returns_plain_text_with_timestamps() -> anyhow::Result<()> {
    let app = build_router(test_state().await?);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/logs/export")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let text = std::str::from_utf8(&body)?;
    assert!(text.contains("INFO server trapo-server initialized"));
    assert!(text.lines().all(|line| line.starts_with("20")));
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

fn seed_completed_text_run(
    config: &ServerConfig,
    run_id: &str,
    file_hash: &str,
    text: &str,
) -> anyhow::Result<()> {
    let conn = duckdb::Connection::open(&config.database_path)?;
    let root_path = config.app_root.to_string_lossy().to_string();
    let absolute_path = config
        .app_root
        .join("long-cjk.pdf")
        .to_string_lossy()
        .to_string();
    conn.execute(
        "INSERT INTO ingest_runs(
            run_id, root_path, status, profile_id, engine_id, queued_files,
            processed_pages, total_pages, model_id, runtime_id
         )
         VALUES (?, ?, 'completed', 'test-profile', 'test-engine', 1, 1, 1, 'test-model', 'test-runtime')",
        params![run_id, root_path.as_str()],
    )?;
    conn.execute(
        "INSERT INTO files(file_hash, display_name, extension, size_bytes, page_count, status)
         VALUES (?, 'long-cjk.pdf', 'pdf', 10, 1, 'completed')",
        params![file_hash],
    )?;
    conn.execute(
        "INSERT INTO file_locations(file_hash, root_path, absolute_path, relative_path)
         VALUES (?, ?, ?, 'long-cjk.pdf')",
        params![file_hash, root_path.as_str(), absolute_path.as_str()],
    )?;
    conn.execute(
        "INSERT INTO ingest_run_documents(run_id, file_hash, ordinal) VALUES (?, ?, 0)",
        params![run_id, file_hash],
    )?;
    conn.execute(
        "INSERT INTO document_pages(file_hash, page_no, width_px, height_px, render_dpi, status)
         VALUES (?, 1, 100, 100, 200, 'completed')",
        params![file_hash],
    )?;
    conn.execute(
        "INSERT INTO document_run_page_ocr(
            run_id, file_hash, page_no, engine_id, profile_id, raw_text, cleaned_text,
            status, attempts, elapsed_ms, options
         )
         VALUES (?, ?, 1, 'test-engine', 'test-profile', ?, ?, 'completed', 1, 1, '{}'::JSON)",
        params![run_id, file_hash, text, text],
    )?;
    Ok(())
}

struct DiagnosticWaterfallSeed<'a> {
    run: &'a str,
    task: &'a str,
    text_task: &'a str,
    work_unit: &'a str,
    orphan_work_unit: &'a str,
    root_span: &'a str,
    page_span: &'a str,
    text_root_span: &'a str,
    text_page_span: &'a str,
    render_span: &'a str,
    ocr_work_unit: &'a str,
    ocr_span: &'a str,
}

fn seed_diagnostic_waterfall_run(
    config: &ServerConfig,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    let conn = duckdb::Connection::open(&config.database_path)?;
    seed_diagnostic_waterfall_core(&conn, config, seed)?;
    seed_diagnostic_waterfall_work_units(&conn, seed)?;
    seed_diagnostic_waterfall_spans(&conn, seed)?;
    Ok(())
}

fn seed_diagnostic_waterfall_core(
    conn: &duckdb::Connection,
    config: &ServerConfig,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    let root_path = config.app_root.to_string_lossy().to_string();
    let absolute_path = config
        .app_root
        .join("trace.pdf")
        .to_string_lossy()
        .to_string();
    conn.execute(
        "INSERT INTO ingest_runs(
            run_id, root_path, status, profile_id, engine_id, queued_files,
            processed_pages, total_pages, model_id, runtime_id
         )
         VALUES (?, ?, 'completed', 'test-profile', 'test-engine', 1, 1, 1, 'test-model', 'test-runtime')",
        params![seed.run, root_path.as_str()],
    )?;
    conn.execute(
        "INSERT INTO files(file_hash, display_name, extension, size_bytes, page_count, status)
         VALUES ('file-waterfall', 'trace.pdf', 'pdf', 10, 1, 'completed')",
        [],
    )?;
    conn.execute(
        "INSERT INTO file_locations(file_hash, root_path, absolute_path, relative_path)
         VALUES ('file-waterfall', ?, ?, 'trace.pdf')",
        params![root_path.as_str(), absolute_path.as_str()],
    )?;
    conn.execute(
        "INSERT INTO pipeline_tasks(
            task_id, task_kind, origin_run_id, status, params_json, result_json,
            queued_at, started_at, finished_at, runner_id
         )
         VALUES (?, 'generate_embedding', ?, 'completed', '{}'::JSON, '{}'::JSON,
            '2026-07-07T00:00:00Z', '2026-07-07T00:00:01Z',
            '2026-07-07T00:00:05Z', 'test-runner')",
        params![seed.task, seed.run],
    )?;
    conn.execute(
        "INSERT INTO pipeline_tasks(
            task_id, task_kind, origin_run_id, status, params_json, result_json,
            queued_at, started_at, finished_at, runner_id
         )
         VALUES (?, 'text_index', ?, 'completed', '{}'::JSON, '{}'::JSON,
            '2026-07-07T00:00:00Z', '2026-07-07T00:00:00Z',
            '2026-07-07T00:00:01Z', 'test-runner')",
        params![seed.text_task, seed.run],
    )?;
    Ok(())
}

fn seed_diagnostic_waterfall_work_units(
    conn: &duckdb::Connection,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO ingest_work_units(
            work_unit_id, run_id, file_hash, page_no, status, queued_at, started_at, finished_at,
            work_key, phase, engine, provider, model, profile, execution_key,
            artifact_variant, attempt_count, duration_ms, result_json, metadata_json
         )
         VALUES (?, ?, 'file-waterfall', 1, 'completed',
            '2026-07-07 00:00:01', '2026-07-07 00:00:02', '2026-07-07 00:00:04',
            'file-waterfall:1:embedding', 'generate_embedding', 'llama.cpp',
            'local', 'embedding-gemma-300m', 'default', 'embedding', 'page',
            1, 2000, '{}'::JSON, '{}'::JSON)",
        params![seed.work_unit, seed.run],
    )?;
    conn.execute(
        "INSERT INTO ingest_work_units(
            work_unit_id, run_id, file_hash, page_no, status, queued_at, started_at, finished_at,
            work_key, phase, engine, provider, model, profile, execution_key,
            artifact_variant, attempt_count, duration_ms, result_json, metadata_json
         )
         VALUES (?, ?, 'file-waterfall', 2, 'completed',
            '2026-07-07 00:00:02', '2026-07-07 00:00:03', '2026-07-07 00:00:04',
            'file-waterfall:2:embedding', 'generate_embedding', 'llama.cpp',
            'local', 'embedding-gemma-300m', 'default', 'embedding', 'page',
            1, 1000, '{}'::JSON, '{}'::JSON)",
        params![seed.orphan_work_unit, seed.run],
    )?;
    conn.execute(
        "INSERT INTO ingest_work_units(
            work_unit_id, run_id, file_hash, page_no, status, queued_at, started_at, finished_at,
            work_key, phase, engine, provider, model, profile, execution_key,
            artifact_variant, attempt_count, duration_ms, result_json, metadata_json
         )
         VALUES (?, ?, 'file-waterfall', 3, 'completed',
            '2026-07-07 00:00:01', '2026-07-07 00:00:02', '2026-07-07 00:00:03',
            'file-waterfall:3:ocr', 'ocr', 'unlimited-ocr-ffi',
            'local', 'ocr-model', 'default', 'ocr', 'page',
            1, 1000, '{}'::JSON, '{}'::JSON)",
        params![seed.ocr_work_unit, seed.run],
    )?;
    Ok(())
}

fn seed_diagnostic_waterfall_spans(
    conn: &duckdb::Connection,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    seed_embedding_waterfall_spans(conn, seed)?;
    seed_text_index_waterfall_spans(conn, seed)?;
    seed_ocr_waterfall_spans(conn, seed)?;
    Ok(())
}

fn seed_embedding_waterfall_spans(
    conn: &duckdb::Connection,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, NULL, 'Generate embedding', '2026-07-07 00:00:01',
            '2026-07-07 00:00:05', '{}'::JSON, ?, NULL, NULL,
            'generate_embedding', 'rag', 'ok', '2026-07-07T00:00:05Z',
            4000, '{}'::JSON, ?, NULL, 'task')",
        params![seed.root_span, seed.run, seed.run, seed.task],
    )?;
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, ?, 'Embed page', '2026-07-07 00:00:02',
            '2026-07-07 00:00:04', '{}'::JSON, ?, 'file-waterfall', 1,
            'generate_embedding', 'rag', 'ok', '2026-07-07T00:00:04Z',
            2000, '{}'::JSON, ?, ?, 'embedding_page')",
        params![
            seed.page_span,
            seed.run,
            seed.root_span,
            seed.run,
            seed.task,
            seed.work_unit
        ],
    )?;
    Ok(())
}

fn seed_text_index_waterfall_spans(
    conn: &duckdb::Connection,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, NULL, 'Text index', '2026-07-07 00:00:00',
            '2026-07-07 00:00:01', '{}'::JSON, ?, NULL, NULL,
            'text_index', 'rag', 'ok', '2026-07-07T00:00:01Z',
            1000, '{}'::JSON, ?, NULL, 'task')",
        params![seed.text_root_span, seed.run, seed.run, seed.text_task],
    )?;
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, ?, 'Segment page text', '2026-07-07 00:00:00',
            '2026-07-07 00:00:01', '{}'::JSON, ?, 'file-waterfall', 1,
            'text_index', 'text_page', 'ok', '2026-07-07T00:00:01Z',
            1000, '{}'::JSON, ?, NULL, 'text_page')",
        params![
            seed.text_page_span,
            seed.run,
            seed.text_root_span,
            seed.run,
            seed.text_task
        ],
    )?;
    Ok(())
}

fn seed_ocr_waterfall_spans(
    conn: &duckdb::Connection,
    seed: &DiagnosticWaterfallSeed<'_>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, NULL, 'Render document', '2026-07-07 00:00:01',
            '2026-07-07 00:00:02', '{}'::JSON, ?, 'file-waterfall', NULL,
            'render', 'file', 'ok', '2026-07-07T00:00:02Z',
            1000, '{}'::JSON, NULL, NULL, 'file')",
        params![seed.render_span, seed.run, seed.run],
    )?;
    conn.execute(
        "INSERT INTO ingest_diagnostic_spans(
            span_id, run_id, parent_span_id, name, started_at, finished_at, attributes,
            trace_id, file_hash, page_no, pipeline_step, category, status, ended_at,
            duration_ms, attributes_json, task_id, work_unit_id, span_kind
         )
         VALUES (?, ?, NULL, 'OCR page', '2026-07-07 00:00:02',
            '2026-07-07 00:00:03', '{}'::JSON, ?, 'file-waterfall', 3,
            'ocr', 'page', 'ok', '2026-07-07T00:00:03Z',
            1000, '{}'::JSON, NULL, ?, 'page')",
        params![seed.ocr_span, seed.run, seed.run, seed.ocr_work_unit],
    )?;
    Ok(())
}

async fn test_state() -> anyhow::Result<AppState> {
    let temp = tempfile::tempdir()?;
    let root = temp.keep();
    AppState::new(test_config(&root)?).await.map_err(Into::into)
}

fn test_config(root: &std::path::Path) -> anyhow::Result<ServerConfig> {
    let client_dist = root.join("src").join("trapo-client").join("dist");
    std::fs::create_dir_all(&client_dist)?;
    std::fs::write(client_dist.join("index.html"), "<!doctype html>")?;
    install_placeholder_cpu_runtime(root)?;
    install_placeholder_default_model(root)?;
    Ok(ServerConfig {
        app_root: root.to_path_buf(),
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
}

fn install_placeholder_default_model(root: &std::path::Path) -> anyhow::Result<()> {
    let model_dir = root.join("models");
    std::fs::create_dir_all(&model_dir)?;
    std::fs::write(model_dir.join("Unlimited-OCR-Q4_K_M.gguf"), b"model")?;
    std::fs::write(model_dir.join("mmproj-Unlimited-OCR-F16.gguf"), b"mmproj")?;
    Ok(())
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
