use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::{backup::Metadata, ClientArc};
use fedimint_core::config::FederationId;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BackupRequest {
    pub metadata: BTreeMap<String, String>,
    pub federation_id: Option<FederationId>,
}

async fn _backup(client: ClientArc, req: BackupRequest) -> Result<(), AppError> {
    client
        .backup_to_federation(Metadata::from_json_serialized(req.metadata))
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<BackupRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    _backup(client, v).await?;
    Ok(json!(()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<BackupRequest>,
) -> Result<Json<()>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    _backup(client, req).await?;
    Ok(Json(()))
}
