use std::collections::HashMap;

use axum::extract::State;
use axum::Json;
use multimint::MultiMint;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

async fn _discover_version(multimint: MultiMint) -> Result<Value, AppError> {
    let mut api_versions = HashMap::new();
    for (id, client) in multimint.clients.lock().await.iter() {
        api_versions.insert(
            *id,
            json!({"version" : client.discover_common_api_version().await?}),
        );
    }
    Ok(json!(api_versions))
}

pub async fn handle_ws(state: AppState) -> Result<Value, AppError> {
    let version = _discover_version(state.multimint).await?;
    let version_json = json!(version);
    Ok(version_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let version = _discover_version(state.multimint).await?;
    Ok(Json(version))
}
