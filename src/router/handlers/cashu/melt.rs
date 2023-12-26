use axum::extract::State;

use crate::{error::AppError, state::AppState};

#[axum_macros::debug_handler]
pub async fn handle_melt_quote(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_melt_quote_quote_id(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_melt(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}
