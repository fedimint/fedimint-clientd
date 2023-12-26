use std::time::Duration;

use axum::{extract::State, Json};
use fedimint_core::time::now;
use fedimint_wallet_client::WalletClientModule;

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{DepositAddressRequest, DepositAddressResponse},
};

#[axum_macros::debug_handler]
pub async fn handle_depositaddress(
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
