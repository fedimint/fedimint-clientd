use axum::{extract::State, Json};
use serde_json::Value;

use crate::{error::AppError, state::AppState};

#[axum_macros::debug_handler]
pub async fn handle_config(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let config = state.fm.get_config_json();
    Ok(Json(serde_json::to_value(config).expect("Client config is serializable")))
}
