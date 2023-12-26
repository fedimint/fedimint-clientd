use anyhow::Context;
use axum::{extract::State, Json};
use fedimint_core::{core::OperationId, Amount};
use fedimint_ln_client::{LightningClientModule, OutgoingLightningPayment, PayType};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    error::AppError,
    router::handlers::fedimint::ln::{get_invoice, wait_for_ln_payment},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct LnPayRequest {
    pub payment_info: String,
    pub amount_msat: Option<Amount>,
    pub finish_in_background: bool,
    pub lnurl_comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LnPayResponse {
    pub operation_id: OperationId,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: Amount,
}

#[axum_macros::debug_handler]
pub async fn handle_pay(
    State(state): State<AppState>,
    Json(req): Json<LnPayRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let bolt11 = get_invoice(&req).await?;
    info!("Paying invoice: {bolt11}");
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    // let gateway = lightning_module.select_active_gateway_opt().await;
    let OutgoingLightningPayment {
        payment_type,
        contract_id,
        fee,
    } = lightning_module.pay_bolt11_invoice(bolt11, ()).await?;
    let operation_id = payment_type.operation_id();
    info!("Gateway fee: {fee}, payment operation id: {operation_id}");
    if req.finish_in_background {
        wait_for_ln_payment(&state.fm, payment_type, contract_id.to_string(), true).await?;
        info!("Payment will finish in background, use await-ln-pay to get the result");
        Ok(Json(LnPayResponse {
            operation_id,
            payment_type: payment_type,
            contract_id: contract_id.to_string(),
            fee,
        }))
    } else {
        Ok(Json(
            wait_for_ln_payment(&state.fm, payment_type, contract_id.to_string(), false)
                .await?
                .context("expected a response")?,
        ))
    }
}
