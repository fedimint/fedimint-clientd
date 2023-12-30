use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_core::core::{ModuleInstanceId, ModuleKind};
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
}

async fn _module(_state: AppState, _req: ModuleRequest) -> Result<(), AppError> {
    // TODO: Figure out how to impl this
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let module = _module(state, v).await?;
    let module_json = json!(module);
    Ok(module_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ModuleRequest>,
) -> Result<Json<()>, AppError> {
    let module = _module(state, req).await?;
    Ok(Json(module))
}
