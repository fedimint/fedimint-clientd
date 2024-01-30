use crate::error::AppError;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use fedimint_client::ClientArc;
use serde_json::{json, Value};

use crate::state::AppState;

async fn _restore(_client: ClientArc, _v: Value) -> Result<(), AppError> {
    // TODO: unimplemented in cli
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let client = state.get_client(None).await?;
    let restore = _restore(client, v).await?;
    let restore_json = json!(restore);
    Ok(restore_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<Value>,
) -> Result<Json<()>, AppError> {
    let client = state.get_client(None).await?;
    let restore = _restore(client, req).await?;
    Ok(Json(restore))
}
