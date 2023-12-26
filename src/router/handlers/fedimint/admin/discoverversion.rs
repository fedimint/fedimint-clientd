use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[axum_macros::debug_handler]
pub async fn handle_discoverversion(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "version": state.fm.discover_common_api_version().await? })))
}
