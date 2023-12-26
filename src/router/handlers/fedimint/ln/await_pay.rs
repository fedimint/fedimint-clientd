use anyhow::Context;
use axum::{extract::State, Json};
use fedimint_ln_client::{LightningClientModule, PayType};

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{AwaitInvoiceRequest, LnPayResponse},
    utils::wait_for_ln_payment,
};

#[axum_macros::debug_handler]
pub async fn handle_await_pay(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    let ln_pay_details = lightning_module
        .get_ln_pay_details_for(req.operation_id)
        .await?;
    let payment_type = if ln_pay_details.is_internal_payment {
        PayType::Internal(req.operation_id)
    } else {
        PayType::Lightning(req.operation_id)
    };
    Ok(Json(
        wait_for_ln_payment(
            &state.fm,
            payment_type,
            ln_pay_details.contract_id.to_string(),
            false,
        )
        .await?
        .context("expected a response")?,
    ))
}
