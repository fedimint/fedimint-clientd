use std::time::Duration;

use crate::{
    error::AppError,
    router::handlers::cashu::{Method, Unit},
    state::AppState,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use fedimint_client::ClientArc;
use fedimint_core::{time::now, Amount};
use fedimint_ln_client::LightningClientModule;
use fedimint_wallet_client::WalletClientModule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PostMintQuoteMethodRequest {
    pub amount: Amount,
    pub unit: Unit,
}

#[derive(Debug, Serialize)]
pub struct PostMintQuoteMethodResponse {
    pub quote: String,
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
    let res = match method {
        Method::Bolt11 => match req.unit {
            Unit::Msat => mint_bolt11(state.fm, req.amount).await,
            Unit::Sat => mint_bolt11(state.fm, req.amount * 1000).await,
        },
        Method::Onchain => match req.unit {
            Unit::Msat => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("Unsupported unit for onchain mint, use sat instead"),
            )),
            Unit::Sat => mint_onchain(state.fm, req.amount * 1000).await,
        },
    }?;

    Ok(Json(res))
}

const DEFAULT_MINT_EXPIRY_OFFSET: u64 = 3600;
const DEFAULT_MINT_DESCRIPTION: &str = "Cashu mint operation";

pub async fn mint_bolt11(
    client: ClientArc,
    amount_msat: Amount,
) -> Result<PostMintQuoteMethodResponse, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let valid_until = now() + Duration::from_secs(DEFAULT_MINT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;

    let (operation_id, invoice) =
        lightning_module
            .create_bolt11_invoice(
                amount_msat,
                format!("{}, method={:?}", DEFAULT_MINT_DESCRIPTION, Method::Bolt11),
                Some(expiry_time),
                (),
            )
            .await?;

    Ok(PostMintQuoteMethodResponse {
        quote: operation_id.to_string(),
        request: invoice.to_string(),
        paid: false,
        expiry: expiry_time.try_into()?,
    })
}

async fn mint_onchain(
    client: ClientArc,
    _amount_sat: Amount,
) -> Result<PostMintQuoteMethodResponse, AppError> {
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let valid_until = now() + Duration::from_secs(DEFAULT_MINT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;

    let (operation_id, address) = wallet_client.get_deposit_address(valid_until, ()).await?;

    Ok(PostMintQuoteMethodResponse {
        quote: operation_id.to_string(),
        request: address.to_string(),
        paid: false,
        expiry: expiry_time.try_into()?,
    })
}
