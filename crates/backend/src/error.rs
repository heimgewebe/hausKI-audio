use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("startup failed: {0}")]
    Startup(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Upstream(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn upstream(message: impl Into<String>) -> Self {
        Self::Upstream(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message.as_str()),
            AppError::Startup(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.as_str()),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message.as_str()),
            AppError::Upstream(message) => (StatusCode::BAD_GATEWAY, message.as_str()),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.as_str()),
        };

        let payload = json!({
            "error": message,
        });

        (status, Json(payload)).into_response()
    }
}
