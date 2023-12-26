use std::{str::FromStr, time::Duration};

use crate::{
    error::AppError,
    router::handlers::cashu::{Method, Unit},
    state::AppState,
};
use anyhow::{anyhow, bail};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use fedimint_client::ClientArc;
use fedimint_core::{time::now, Amount};
use fedimint_ln_client::{LightningClientModule, OutgoingLightningPayment};
use fedimint_wallet_client::WalletClientModule;
use lightning_invoice::Bolt11Invoice;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct PostMeltQuoteMethodRequest {
    pub request: String,
    pub amount: Amount,
    pub unit: Unit,
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
    let res = match method {
        Method::Bolt11 => match req.unit {
            Unit::Msat => melt_bolt11(state.fm, req.request, req.amount).await,
            Unit::Sat => melt_bolt11(state.fm, req.request, req.amount * 1000).await,
        },
        Method::Onchain => match req.unit {
            Unit::Msat => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("Unsupported unit for onchain melt, use sat instead"),
            )),
            Unit::Sat => melt_onchain(state.fm, req.amount * 1000).await,
        },
    }?;

    Ok(Json(res))
}

const DEFAULT_MELT_EXPIRY_OFFSET: u64 = 3600;
const DEFAULT_MELT_DESCRIPTION: &str = "Cashu melt operation";

pub async fn melt_bolt11(
    client: ClientArc,
    request: String,
    amount_msat: Amount,
) -> Result<PostMeltQuoteMethodResponse, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

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
    } = lightning_module.pay_bolt11_invoice(bolt11, ()).await?;

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
    client: ClientArc,
    amount_sat: Amount,
) -> Result<PostMeltQuoteMethodResponse, AppError> {
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let valid_until = now() + Duration::from_secs(DEFAULT_MELT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;

    let (operation_id, address) = wallet_client.get_deposit_address(valid_until, ()).await?;

    Ok(PostMeltQuoteMethodResponse {
        quote: operation_id.to_string(),
        amount: amount_sat,
        fee_reserve: Amount::from_sats(0), // Fedimint doesn't charge fees
        paid: false,
        expiry: expiry_time.try_into()?,
    })
}
