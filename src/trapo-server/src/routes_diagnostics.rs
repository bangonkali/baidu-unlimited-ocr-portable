#[derive(Debug, Deserialize)]
struct OcrEventsQuery {
    run_id: Option<String>,
    file_hash: Option<String>,
    page_no: Option<u32>,
    since_sequence: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticRunQuery {
    run_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticTraceQuery {
    run_id: Option<String>,
    file_hash: Option<String>,
    page_no: Option<u32>,
    status: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
}

async fn ocr_events(
    State(state): State<AppState>,
    Query(query): Query<OcrEventsQuery>,
) -> Result<Json<crate::workbench_types::OcrReplayPayload>> {
    Ok(Json(
        state
            .ocr_replay(
                query.run_id,
                query.file_hash,
                query.page_no,
                query.since_sequence,
                query.limit.unwrap_or(5_000),
            )
            .await?,
    ))
}

async fn diagnostics_runs(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticRunsPayload>> {
    Ok(Json(state.diagnostics_runs(query.limit.unwrap_or(100)).await?))
}

async fn diagnostics_trace(
    State(state): State<AppState>,
    Query(query): Query<DiagnosticTraceQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticTracePayload>> {
    Ok(Json(
        state
            .diagnostic_trace(crate::app::DiagnosticTraceRequest {
                run_id: query.run_id,
                file_hash: query.file_hash,
                page_no: query.page_no,
                status: query.status,
                q: query.q,
                limit: query.limit.unwrap_or(5_000),
            })
            .await?,
    ))
}

async fn diagnostics_waterfall(
    State(state): State<AppState>,
    Query(query): Query<DiagnosticTraceQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticWaterfallPayload>> {
    Ok(Json(
        state
            .diagnostic_waterfall(crate::app::DiagnosticTraceRequest {
                run_id: query.run_id,
                file_hash: query.file_hash,
                page_no: query.page_no,
                status: query.status,
                q: query.q,
                limit: query.limit.unwrap_or(10_000),
            })
            .await?,
    ))
}

async fn diagnostics_progress(
    State(state): State<AppState>,
    Query(query): Query<DiagnosticRunQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticProgressPayload>> {
    Ok(Json(
        state
            .diagnostic_progress(query.run_id, query.limit.unwrap_or(5_000))
            .await?,
    ))
}

async fn diagnostics_analytics(
    State(state): State<AppState>,
    Query(query): Query<DiagnosticRunQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticAnalyticsPayload>> {
    Ok(Json(
        state
            .diagnostic_analytics(query.run_id, query.limit.unwrap_or(10_000))
            .await?,
    ))
}

async fn diagnostics_models(
    State(state): State<AppState>,
    Query(query): Query<DiagnosticRunQuery>,
) -> Result<Json<crate::workbench_types::DiagnosticModelsPayload>> {
    Ok(Json(
        state
            .diagnostic_models(query.run_id, query.limit.unwrap_or(1_000))
            .await?,
    ))
}
