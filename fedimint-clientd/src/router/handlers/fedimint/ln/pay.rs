use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bitcoin::secp256k1::PublicKey;
use fedimint_client::ClientHandleArc;
use fedimint_core::config::FederationId;
use fedimint_core::core::OperationId;
use fedimint_core::Amount;
use fedimint_ln_client::{LightningClientModule, OutgoingLightningPayment, PayType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::error::AppError;
use crate::router::handlers::fedimint::ln::{get_invoice, wait_for_ln_payment};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LnPayRequest {
    pub payment_info: String,
    pub amount_msat: Option<Amount>,
    pub lnurl_comment: Option<String>,
    pub gateway_id: PublicKey,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LnPayResponse {
    pub operation_id: OperationId,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: Amount,
}

async fn _pay(client: ClientHandleArc, req: LnPayRequest) -> Result<LnPayResponse, AppError> {
    let bolt11 = get_invoice(&req).await?;
    info!("Paying invoice: {bolt11}");
    let lightning_module = client.get_first_module::<LightningClientModule>();
    let gateway = lightning_module
        .select_gateway(&req.gateway_id)
        .await
        .ok_or_else(|| {
            error!("Failed to select gateway: {}", req.gateway_id);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("Failed to select gateway"),
            )
        })?;

    let OutgoingLightningPayment {
        payment_type,
        contract_id,
        fee,
    } = lightning_module
        .pay_bolt11_invoice(Some(gateway), bolt11, ())
        .await?;
    let operation_id = payment_type.operation_id();
    info!("Gateway fee: {fee}, payment operation id: {operation_id}");

    wait_for_ln_payment(&client, payment_type, contract_id.to_string(), false)
        .await?
        .ok_or_else(|| {
            error!("Payment failed");
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("Payment failed"))
        })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<LnPayRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let pay = _pay(client, v).await?;
    let pay_json = json!(pay);
    Ok(pay_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<LnPayRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let pay = _pay(client, req).await?;
    Ok(Json(pay))
}
