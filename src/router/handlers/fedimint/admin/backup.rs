use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::backup::Metadata;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub metadata: BTreeMap<String, String>,
}

async fn _backup(state: AppState, req: BackupRequest) -> Result<(), AppError> {
    state
        .fm
        .backup_to_federation(Metadata::from_json_serialized(req.metadata))
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value::<BackupRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let backup = _backup(state, v).await?;
    let backup_json = json!(backup);
    Ok(backup_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<BackupRequest>,
) -> Result<Json<()>, AppError> {
    let backup = _backup(state, req).await?;
    Ok(Json(backup))
}
