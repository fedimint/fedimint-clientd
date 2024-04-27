use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_client::ClientHandleArc;
use fedimint_core::config::FederationId;
use fedimint_ln_client::LightningClientModule;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGatewaysRequest {
    pub federation_id: FederationId,
}

async fn _list_gateways(client: ClientHandleArc) -> Result<Value, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    let gateways = lightning_module.list_gateways().await;
    if gateways.is_empty() {
        return Ok(serde_json::to_value(Vec::<String>::new())?);
    }

    let mut gateways_json = json!(&gateways);
    let gateways_json_array = gateways_json
        .as_array_mut()
        .ok_or_else(|| anyhow!("gateways_json is not an array"))?;

    for gateway in gateways_json_array.iter_mut() {
        let gateway_obj = gateway
            .as_object_mut()
            .ok_or_else(|| anyhow!("gateway is not an object"))?;
        gateway_obj.insert("federation_id".to_string(), json!(client.federation_id()));
    }

    Ok(serde_json::to_value(gateways_json_array)?)
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<ListGatewaysRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let gateways = _list_gateways(client).await?;
    let gateways_json = json!(gateways);
    Ok(gateways_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ListGatewaysRequest>,
) -> Result<Json<Value>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let gateways = _list_gateways(client).await?;
    Ok(Json(gateways))
}
