use axum::{extract::State, Json};
use fedimint_client::ClientArc;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

async fn _config(client: ClientArc) -> Result<Value, AppError> {
    let config = client.get_config_json();
    Ok(serde_json::to_value(config).expect("Client config is serializable"))
}

pub async fn handle_ws(state: AppState) -> Result<Value, AppError> {
    let client = state.get_client(None).await?;
    let config = _config(client).await?;
    let config_json = json!(config);
    Ok(config_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let client = state.get_client(None).await?;
    let config = _config(client).await?;
    Ok(Json(config))
}
