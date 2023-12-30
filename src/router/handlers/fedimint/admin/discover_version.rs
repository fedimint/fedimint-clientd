use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

async fn _discover_version(state: AppState) -> Result<Value, AppError> {
    let version = state.fm.discover_common_api_version().await?;
    Ok(json!({ "version": version }))
}

pub async fn handle_ws(state: AppState) -> Result<Value, AppError> {
    let version = _discover_version(state).await?;
    let version_json = json!(version);
    Ok(version_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let version = _discover_version(state).await?;
    Ok(Json(version))
}
