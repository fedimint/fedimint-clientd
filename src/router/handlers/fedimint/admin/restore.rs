use crate::error::AppError;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

async fn _restore(_v: Value, _state: AppState) -> Result<(), AppError> {
    // TODO: unimplemented in cli
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let restore = _restore(v, state).await?;
    let restore_json = json!(restore);
    Ok(restore_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<Value>,
) -> Result<Json<()>, AppError> {
    let restore = _restore(req, state).await?;
    Ok(Json(restore))
}
