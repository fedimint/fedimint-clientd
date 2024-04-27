use axum::extract::State;

use crate::error::AppError;
use crate::state::AppState;

#[axum_macros::debug_handler]
pub async fn handle_keysets(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}
