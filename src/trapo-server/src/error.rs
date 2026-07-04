use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// Result type used by the Trapo server library API.
pub type Result<T> = std::result::Result<T, AppError>;

/// Error variants returned by server initialization and request handlers.
#[derive(Debug, Error)]
pub enum AppError {
    /// Request validation failed.
    #[error("{0}")]
    BadRequest(String),
    /// A requested resource was not found.
    #[error("{0}")]
    NotFound(String),
    /// The request conflicts with current server state.
    #[error("{0}")]
    Conflict(String),
    /// `DuckDB` returned an error.
    #[error("database error: {0}")]
    Database(#[from] duckdb::Error),
    /// Filesystem or OS I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization or parsing failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Internal server failure with a caller-safe message.
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorPayload {
    pub(crate) error: String,
}

impl AppError {
    /// Returns the HTTP status code associated with this error.
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Io(_) | Self::Json(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = Json(ErrorPayload {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}
