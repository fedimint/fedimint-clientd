use crate::{error::AppError, state::AppState};
use axum::{extract::State, Json};
use bitcoin::Address;
use fedimint_core::{core::OperationId, time::now};
use fedimint_wallet_client::WalletClientModule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct DepositAddressRequest {
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct DepositAddressResponse {
    pub address: Address,
    pub operation_id: OperationId,
}

async fn _deposit_address(
    state: AppState,
    req: DepositAddressRequest,
) -> Result<DepositAddressResponse, AppError> {
    let wallet_module = state.fm.get_first_module::<WalletClientModule>();
    let (operation_id, address) = wallet_module
        .get_deposit_address(now() + Duration::from_secs(req.timeout), ())
        .await?;

    Ok(DepositAddressResponse {
        address,
        operation_id,
    })
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v: DepositAddressRequest = serde_json::from_value(v).unwrap();
    let withdraw = _deposit_address(state, v).await?;
    let withdraw_json = json!(withdraw);
    Ok(withdraw_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<DepositAddressRequest>,
) -> Result<Json<DepositAddressResponse>, AppError> {
    let withdraw = _deposit_address(state, req).await?;
    Ok(Json(withdraw))
}
