use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use crate::config::ConfigError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to load configuration: {0}")]
    Config(#[from] ConfigError),
    #[error("request to Mopidy failed: {0}")]
    Mopidy(#[from] reqwest::Error),
    #[error("failed to execute command: {0}")]
    Command(#[from] std::io::Error),
    #[error("command returned non-zero exit code: {0}")]
    CommandStatus(String),
    #[error("configuration validation failed: {0}")]
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
        let status = match &self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Mopidy(_) | AppError::Upstream(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = json!({
            "error": self.to_string(),
        });

        (status, Json(payload)).into_response()
    }
}
