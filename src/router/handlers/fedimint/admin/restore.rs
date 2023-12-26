use crate::error::AppError;
use anyhow::anyhow;
use axum::http::StatusCode;

#[axum_macros::debug_handler]
pub async fn handle_restore() -> Result<(), AppError> {
    // TODO: unimplemented in cli
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
}
