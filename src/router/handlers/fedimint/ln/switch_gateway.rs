use std::str::FromStr;

use axum::{extract::State, Json};
use bitcoin::secp256k1::PublicKey;
use fedimint_ln_client::LightningClientModule;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SwitchGatewayRequest {
    pub gateway_id: String,
}

#[axum_macros::debug_handler]
pub async fn handle_switch_gateway(
    State(state): State<AppState>,
    Json(req): Json<SwitchGatewayRequest>,
) -> Result<Json<Value>, AppError> {
    let public_key = PublicKey::from_str(&req.gateway_id)?;
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.set_active_gateway(&public_key).await?;
    let gateway = lightning_module.select_active_gateway().await?;
    let mut gateway_json = json!(&gateway);
    gateway_json["active"] = json!(true);
    Ok(Json(serde_json::to_value(gateway_json).unwrap()))
}
