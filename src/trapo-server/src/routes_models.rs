async fn models(State(state): State<AppState>) -> Json<crate::types::ModelsPayload> {
    Json(state.models().await)
}

async fn download_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Json(request): Json<crate::types::ModelDownloadRequest>,
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

async fn model_events(State(state): State<AppState>, Path(model_id): Path<String>) -> Result<Response> {
    let record = state.model_download_event(&model_id).await?;
    let data = serde_json::to_string(&record)?;
    Ok((
        [(header::CONTENT_TYPE, "text/event-stream")],
        format!("event: model\ndata: {data}\n\n"),
    )
        .into_response())
}
