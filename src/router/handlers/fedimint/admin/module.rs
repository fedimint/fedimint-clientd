use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::ClientArc;
use fedimint_core::{config::FederationId, core::{ModuleInstanceId, ModuleKind}};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, Deserialize)]
pub enum ModuleSelector {
    Id(ModuleInstanceId),
    Kind(ModuleKind),
}

#[derive(Debug, Deserialize)]
pub struct ModuleRequest {
    pub module: ModuleSelector,
    pub args: Vec<String>,
    pub federation_id: Option<FederationId>,
}

async fn _module(_client: ClientArc, _req: ModuleRequest) -> Result<(), AppError> {
    // TODO: Figure out how to impl this
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let client = state.get_client(None).await?;
    let v = serde_json::from_value(v).unwrap();
    let module = _module(client, v).await?;
    let module_json = json!(module);
    Ok(module_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ModuleRequest>,
) -> Result<Json<()>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let module = _module(client, req).await?;
    Ok(Json(module))
}
