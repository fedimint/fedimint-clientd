use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_client::ClientArc;
use serde_json::{json, Value};

use crate::error::AppError;
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
    _restore(client, v).await?;
    Ok(json!(()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<Value>,
) -> Result<Json<()>, AppError> {
    let client = state.get_client(None).await?;
    _restore(client, req).await?;
    Ok(Json(()))
}
