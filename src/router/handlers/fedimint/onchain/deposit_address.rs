use std::time::Duration;

use axum::{extract::State, Json};
use bitcoin::Address;
use fedimint_core::{core::OperationId, time::now};
use fedimint_wallet_client::WalletClientModule;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DepositAddressRequest {
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct DepositAddressResponse {
    pub operation_id: OperationId,
    pub address: Address,
}

#[axum_macros::debug_handler]
pub async fn handle_deposit_address(
    State(state): State<AppState>,
    Json(req): Json<DepositAddressRequest>,
) -> Result<Json<DepositAddressResponse>, AppError> {
    let wallet_client = state.fm.get_first_module::<WalletClientModule>();
    let (operation_id, address) = wallet_client
        .get_deposit_address(now() + Duration::from_secs(req.timeout), ())
        .await?;
    Ok(Json(DepositAddressResponse {
        operation_id,
        address,
    }))
}
