use std::fmt;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct AppError {
    pub error: anyhow::Error,
    pub status: StatusCode,
}

impl AppError {
    pub fn new(status: StatusCode, error: impl Into<anyhow::Error>) -> Self {
        Self {
            error: error.into(),
            status,
        }
    }
    pub fn from_status_code(status: StatusCode) -> Self {
        let error_message = format!("HTTP error with status: {}", status.as_u16());
        AppError::new(status, anyhow!(error_message))
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, format!("Something went wrong: {}", self.error)).into_response()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_json = json!({
            "error": self.error.to_string(),
            "status": self.status.as_u16(),
        });

        write!(f, "{}", error_json)
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            error: err.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR, // default status code
        }
    }
}
