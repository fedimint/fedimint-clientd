use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::ws::Message, extract::State, http::StatusCode, Json};
use bitcoin::Address;
use bitcoin_hashes::hex::ToHex;
use fedimint_core::BitcoinAmountOrAll;
use fedimint_wallet_client::{WalletClientModule, WithdrawState};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub address: Address,
    pub amount_msat: BitcoinAmountOrAll,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub txid: String,
    pub fees_sat: u64,
}

async fn _withdraw(state: AppState, req: WithdrawRequest) -> Result<WithdrawResponse, AppError> {
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
                return Ok(WithdrawResponse {
                    txid: txid.to_hex(),
                    fees_sat: absolute_fees.to_sat(),
                });
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

pub async fn handle_ws(v: Value, state: AppState) -> Result<Message, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let withdraw = _withdraw(state, v).await?;
    let withdraw_json = json!(withdraw);
    Ok(Message::Text(withdraw_json.to_string()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, AppError> {
    let withdraw = _withdraw(state, req).await?;
    Ok(Json(withdraw))
}
