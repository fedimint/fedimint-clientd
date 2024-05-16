use std::time::Duration;

use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bitcoin::Address;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::core::OperationId;
use multimint::fedimint_core::time::now;
use multimint::fedimint_wallet_client::WalletClientModule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressRequest {
    pub timeout: u64,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressResponse {
    pub address: Address,
    pub operation_id: OperationId,
}

async fn _deposit_address(
    client: ClientHandleArc,
    req: DepositAddressRequest,
) -> Result<DepositAddressResponse, AppError> {
    let wallet_module = client.get_first_module::<WalletClientModule>();
    let (operation_id, address) = wallet_module
        .get_deposit_address(now() + Duration::from_secs(req.timeout), ())
        .await?;

    Ok(DepositAddressResponse {
        address,
        operation_id,
    })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v: DepositAddressRequest = serde_json::from_value::<DepositAddressRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let withdraw = _deposit_address(client, v).await?;
    let withdraw_json = json!(withdraw);
    Ok(withdraw_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<DepositAddressRequest>,
) -> Result<Json<DepositAddressResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let withdraw = _deposit_address(client, req).await?;
    Ok(Json(withdraw))
}
