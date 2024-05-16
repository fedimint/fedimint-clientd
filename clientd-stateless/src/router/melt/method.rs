use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use futures_util::StreamExt;
use lightning_invoice::Bolt11Invoice;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::Amount;
use multimint::fedimint_ln_client::{LightningClientModule, OutgoingLightningPayment};
use multimint::fedimint_wallet_client::{WalletClientModule, WithdrawState};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::cashu::{Method, Unit};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PostMeltQuoteMethodRequest {
    pub request: String,
    pub amount: Amount,
    pub unit: Unit,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
pub struct PostMeltQuoteMethodResponse {
    pub quote: String,
    pub amount: Amount,
    pub fee_reserve: Amount,
    pub paid: bool,
    pub expiry: u64,
}

#[axum_macros::debug_handler]
pub async fn handle_method(
    Path(method): Path<Method>,
    State(state): State<AppState>,
    Json(req): Json<PostMeltQuoteMethodRequest>,
) -> Result<Json<PostMeltQuoteMethodResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let res = match method {
        Method::Bolt11 => match req.unit {
            Unit::Msat => melt_bolt11(client, req.request, req.amount).await,
            Unit::Sat => melt_bolt11(client, req.request, req.amount * 1000).await,
        },
        Method::Onchain => match req.unit {
            Unit::Msat => {
                let amount_sat = bitcoin::Amount::from_sat(req.amount.try_into_sats()?);
                melt_onchain(client, req.request, amount_sat).await
            }
            Unit::Sat => {
                let amount_sat = req.amount * 1000;
                let amount_sat = bitcoin::Amount::from_sat(amount_sat.try_into_sats()?);
                melt_onchain(client, req.request, amount_sat).await
            }
        },
    }?;

    Ok(Json(res))
}

pub async fn melt_bolt11(
    client: ClientHandleArc,
    request: String,
    amount_msat: Amount,
) -> Result<PostMeltQuoteMethodResponse, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    let gateway_id = match lightning_module.list_gateways().await.first() {
        Some(gateway_announcement) => gateway_announcement.info.gateway_id,
        None => {
            error!("No gateways available");
            return Err(AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("No gateways available"),
            ));
        }
    };
    let gateway = lightning_module
        .select_gateway(&gateway_id)
        .await
        .ok_or_else(|| {
            error!("Failed to select gateway");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("Failed to select gateway"),
            )
        })?;

    let bolt11 = Bolt11Invoice::from_str(&request)?;
    let bolt11_amount = Amount::from_msats(
        bolt11
            .amount_milli_satoshis()
            .ok_or_else(|| anyhow!("Cannot pay amountless invoices",))?,
    );

    if bolt11_amount != amount_msat {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!(
                "Invoice amount ({}) does not match request amount ({})",
                bolt11_amount,
                amount_msat
            ),
        ));
    }

    let OutgoingLightningPayment {
        payment_type,
        contract_id: _,
        fee,
    } = lightning_module
        .pay_bolt11_invoice(Some(gateway), bolt11, ())
        .await?;

    let operation_id = payment_type.operation_id();
    info!("Gateway fee: {fee}, payment operation id: {operation_id}");

    Ok(PostMeltQuoteMethodResponse {
        quote: operation_id.to_string(),
        amount: amount_msat,
        fee_reserve: fee,
        paid: false,
        expiry: 0,
    })
}

async fn melt_onchain(
    client: ClientHandleArc,
    request: String,
    amount_sat: bitcoin::Amount,
) -> Result<PostMeltQuoteMethodResponse, AppError> {
    let address = bitcoin::Address::from_str(&request)
        .map_err(|e| anyhow::anyhow!("Onchain request must be a valid bitcoin address: {e}"))?;
    let wallet_module = client.get_first_module::<WalletClientModule>();
    let fees = wallet_module
        .get_withdraw_fees(address.clone(), amount_sat)
        .await?;
    let absolute_fees = fees.amount();

    info!("Attempting withdraw with fees: {fees:?}");

    let operation_id = wallet_module
        .withdraw(address, amount_sat, fees, ())
        .await?;

    let mut updates = wallet_module
        .subscribe_withdraw_updates(operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        info!("Update: {update:?}");

        match update {
            WithdrawState::Succeeded(_txid) => {
                return Ok(PostMeltQuoteMethodResponse {
                    quote: operation_id.to_string(),
                    amount: amount_sat.into(),
                    fee_reserve: absolute_fees.into(),
                    paid: true,
                    expiry: 0,
                });
            }
            WithdrawState::Failed(e) => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!("Onchain melt failed: {:?}", e),
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
