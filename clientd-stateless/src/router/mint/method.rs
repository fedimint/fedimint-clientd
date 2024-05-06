use std::time::Duration;

use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use fedimint_client::ClientHandleArc;
use fedimint_core::config::FederationId;
use fedimint_core::time::now;
use fedimint_core::Amount;
use fedimint_ln_client::LightningClientModule;
use fedimint_wallet_client::WalletClientModule;
use lightning_invoice::{Bolt11InvoiceDescription, Description};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::cashu::{Method, Unit};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMintQuoteMethodRequest {
    pub amount: Amount,
    pub unit: Unit,
    pub federation_id: FederationId,
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
    let client = state.get_client(req.federation_id).await?;
    let res = match method {
        Method::Bolt11 => match req.unit {
            Unit::Msat => mint_bolt11(client, req.amount).await,
            Unit::Sat => mint_bolt11(client, req.amount * 1000).await,
        },
        Method::Onchain => match req.unit {
            Unit::Msat => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("Unsupported unit for onchain mint, use sat instead"),
            )),
            Unit::Sat => mint_onchain(client, req.amount * 1000).await,
        },
    }?;

    Ok(Json(res))
}

const DEFAULT_MINT_EXPIRY_OFFSET: u64 = 3600;
const DEFAULT_MINT_DESCRIPTION: &str = "Cashu mint operation";

pub async fn mint_bolt11(
    client: ClientHandleArc,
    amount_msat: Amount,
) -> Result<PostMintQuoteMethodResponse, AppError> {
    let valid_until = now() + Duration::from_secs(DEFAULT_MINT_EXPIRY_OFFSET);
    let expiry_time = crate::utils::system_time_to_u64(valid_until)?;
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

    let (operation_id, invoice, _) = lightning_module
        .create_bolt11_invoice(
            amount_msat,
            Bolt11InvoiceDescription::Direct(&Description::new(
                DEFAULT_MINT_DESCRIPTION.to_string(),
            )?),
            Some(expiry_time),
            (),
            Some(gateway),
        )
        .await?;

    Ok(PostMintQuoteMethodResponse {
        quote: operation_id.to_string(),
        request: invoice.to_string(),
        paid: false,
        expiry: expiry_time,
    })
}

async fn mint_onchain(
    client: ClientHandleArc,
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
        expiry: expiry_time,
    })
}
