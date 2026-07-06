use axum::http::HeaderMap;

use crate::types::ShutdownRequest;

async fn shutdown(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ShutdownRequest>,
) -> Result<(StatusCode, Json<crate::types::ShutdownPayload>)> {
    if request.confirm != "shutdown"
        || headers
            .get("x-trapo-intent")
            .and_then(|value| value.to_str().ok())
            != Some("shutdown")
    {
        return Err(AppError::BadRequest(
            "shutdown requires confirm=shutdown and x-trapo-intent=shutdown".to_string(),
        ));
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(state.request_shutdown("api").await?),
    ))
}
