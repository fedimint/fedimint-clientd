use axum::{extract::State, Json};
use bitcoin::secp256k1::PublicKey;
use fedimint_ln_client::LightningClientModule;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SwitchGatewayRequest {
    pub gateway_id: String,
}

async fn _switch_gateway(state: AppState, req: SwitchGatewayRequest) -> Result<Value, AppError> {
    let public_key = PublicKey::from_str(&req.gateway_id)?;
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.set_active_gateway(&public_key).await?;
    let gateway = lightning_module.select_active_gateway().await?;
    let mut gateway_json = json!(&gateway);
    gateway_json["active"] = json!(true);
    Ok(serde_json::to_value(gateway_json).unwrap())
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let gateway = _switch_gateway(state, v).await?;
    let gateway_json = json!(gateway);
    Ok(gateway_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<SwitchGatewayRequest>,
) -> Result<Json<Value>, AppError> {
    let gateway = _switch_gateway(state, req).await?;
    Ok(Json(gateway))
}
