use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use multimint::MultiMint;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoverVersionRequest {
    threshold: Option<usize>,
}

async fn _discover_version(
    multimint: MultiMint,
    threshold: Option<usize>,
) -> Result<Value, AppError> {
    let mut api_versions = HashMap::new();
    for (id, client) in multimint.clients.lock().await.iter() {
        api_versions.insert(
            *id,
            json!({"version" : client.discover_common_api_version(threshold).await?}),
        );
    }
    Ok(json!(api_versions))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<DiscoverVersionRequest>(v).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Invalid request: {}", e),
        )
    })?;
    let version = _discover_version(state.multimint, v.threshold).await?;
    let version_json = json!(version);
    Ok(version_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<DiscoverVersionRequest>,
) -> Result<Json<Value>, AppError> {
    let version = _discover_version(state.multimint, req.threshold).await?;
    Ok(Json(version))
}
