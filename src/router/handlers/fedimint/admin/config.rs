use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

async fn _config(state: AppState) -> Result<Value, AppError> {
    let config = state.fm.get_config_json();
    Ok(serde_json::to_value(config).expect("Client config is serializable"))
}

pub async fn handle_ws(state: AppState) -> Result<Value, AppError> {
    let config = _config(state).await?;
    let config_json = json!(config);
    Ok(config_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let config = _config(state).await?;
    Ok(Json(config))
}
