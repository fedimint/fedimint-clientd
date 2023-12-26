use axum::extract::State;

use crate::{error::AppError, state::AppState};

#[axum_macros::debug_handler]
pub async fn handle_method(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}
