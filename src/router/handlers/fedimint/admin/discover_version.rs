use axum::{extract::State, Json};
use fedimint_client::ClientArc;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

async fn _discover_version(client: ClientArc) -> Result<Value, AppError> {
    let version = client.discover_common_api_version().await?;
    Ok(json!({ "version": version }))
}

pub async fn handle_ws(state: AppState) -> Result<Value, AppError> {
    let client = state.get_client(None).await?;
    let version = _discover_version(client).await?;
    let version_json = json!(version);
    Ok(version_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let client = state.get_client(None).await?;
    let version = _discover_version(client).await?;
    Ok(Json(version))
}
