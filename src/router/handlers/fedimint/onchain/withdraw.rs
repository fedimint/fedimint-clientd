use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{WithdrawRequest, WithdrawResponse},
};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use bitcoin_hashes::hex::ToHex;
use fedimint_core::BitcoinAmountOrAll;
use fedimint_wallet_client::{WalletClientModule, WithdrawState};
use futures::StreamExt;
use tracing::info;

#[axum_macros::debug_handler]
pub async fn handle_withdraw(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, AppError> {
    let wallet_module = state.fm.get_first_module::<WalletClientModule>();
    let (amount, fees) = match req.amount_msat {
        // If the amount is "all", then we need to subtract the fees from
        // the amount we are withdrawing
        BitcoinAmountOrAll::All => {
            let balance = bitcoin::Amount::from_sat(state.fm.get_balance().await.msats / 1000);
            let fees = wallet_module
                .get_withdraw_fees(req.address.clone(), balance)
                .await?;
            let amount = balance.checked_sub(fees.amount());
            if amount.is_none() {
                Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    anyhow!("Insufficient balance to pay fees"),
                ))?;
            }
            (amount.unwrap(), fees)
        }
        BitcoinAmountOrAll::Amount(amount) => (
            amount,
            wallet_module
                .get_withdraw_fees(req.address.clone(), amount)
                .await?,
        ),
    };
    let absolute_fees = fees.amount();

    info!("Attempting withdraw with fees: {fees:?}");

    let operation_id = wallet_module
        .withdraw(req.address, amount, fees, ())
        .await?;

    let mut updates = wallet_module
        .subscribe_withdraw_updates(operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        info!("Update: {update:?}");

        match update {
            WithdrawState::Succeeded(txid) => {
                return Ok(Json(WithdrawResponse {
                    txid: txid.to_hex(),
                    fees_sat: absolute_fees.to_sat(),
                }));
            }
            WithdrawState::Failed(e) => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!("Withdraw failed: {:?}", e),
                ));
            }
            _ => continue,
        };
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Update stream ended without outcome"),
    ))
}
