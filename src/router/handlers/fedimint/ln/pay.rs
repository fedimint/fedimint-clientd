use anyhow::Context;
use axum::{extract::State, Json};
use fedimint_ln_client::{LightningClientModule, OutgoingLightningPayment};
use tracing::info;

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{LnPayRequest, LnPayResponse},
    utils::{get_invoice, wait_for_ln_payment},
};

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
