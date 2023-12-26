use std::time::Duration;

use crate::{
    error::AppError,
    router::handlers::cashu::{Method, SupportedUnit},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use fedimint_client::ClientArc;
use fedimint_core::{core::OperationId, time::now, Amount};
use fedimint_ln_client::LightningClientModule;
use fedimint_wallet_client::WalletClientModule;
use serde::{Deserialize, Serialize};
use time::Time;

#[derive(Debug, Deserialize)]
pub struct PostMintQuoteMethodRequest {
    pub amount: Amount,
    pub method: Method,
    pub unit: SupportedUnit,
}

#[derive(Debug, Serialize)]
pub struct PostMintQuoteMethodResponse {
    pub quote: String,
    pub method: Method,
    pub request: String,
    pub paid: bool,
    pub expiry: u64,
}

#[axum_macros::debug_handler]
pub async fn handle_method(
    Path(method): Path<Method>,
    State(state): State<AppState>,
    Json(req): Json<PostMintQuoteMethodRequest>,
) -> Result<Json<PostMintQuoteMethodResponse>, AppError> {
    let amount = match req.unit {
        SupportedUnit::Msat => req.amount,
        SupportedUnit::Sat => req.amount * 1000,
    };

    let res = match method {
        Method::Bolt11 => mint_bolt11(state.fm, amount).await,
        Method::Onchain => mint_onchain(state.fm, amount).await,
    }?;

    Ok(Json(res))
}

const DEFAULT_MINT_EXPIRY_OFFSET: u64 = 3600;
const DEFAULT_MINT_DESCRIPTION: &str = "Cashu mint operation";

pub async fn mint_bolt11(
    client: ClientArc,
    amount: Amount,
) -> Result<PostMintQuoteMethodResponse, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let valid_until = now() + Duration::from_secs(DEFAULT_MINT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;

    let (operation_id, invoice) = lightning_module
        .create_bolt11_invoice(
            amount,
            format!("{}, method={:?}", DEFAULT_MINT_DESCRIPTION, Method::Bolt11),
            Some(expiry_time),
            (),
        )
        .await?;

    Ok(PostMintQuoteMethodResponse {
        quote: operation_id.to_string(),
        method: Method::Bolt11,
        request: invoice.to_string(),
        paid: false,
        expiry: expiry_time.try_into()?,
    })
}

async fn mint_onchain(
    client: ClientArc,
    _amount: Amount,
) -> Result<PostMintQuoteMethodResponse, AppError> {
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let valid_until = now() + Duration::from_secs(DEFAULT_MINT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;

    let (operation_id, address) = wallet_client.get_deposit_address(valid_until, ()).await?;

    Ok(PostMintQuoteMethodResponse {
        quote: operation_id.to_string(),
        method: Method::Onchain,
        request: address.to_string(),
        paid: false,
        expiry: expiry_time.try_into()?,
    })
}
