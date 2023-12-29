use anyhow::Context;
use axum::{extract::ws::Message, extract::State, http::StatusCode, Json};
use fedimint_core::core::OperationId;
use fedimint_ln_client::{LightningClientModule, PayType};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{pay::LnPayResponse, wait_for_ln_payment};
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct AwaitLnPayRequest {
    pub operation_id: OperationId,
}

async fn _await_pay(state: AppState, req: AwaitLnPayRequest) -> Result<LnPayResponse, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    let ln_pay_details = lightning_module
        .get_ln_pay_details_for(req.operation_id)
        .await?;
    let payment_type = if ln_pay_details.is_internal_payment {
        PayType::Internal(req.operation_id)
    } else {
        PayType::Lightning(req.operation_id)
    };
    wait_for_ln_payment(
        &state.fm,
        payment_type,
        ln_pay_details.contract_id.to_string(),
        false,
    )
    .await?
    .context("expected a response")
    .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Message, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let pay = _await_pay(state, v).await?;
    let pay_json = json!(pay);
    Ok(Message::Text(pay_json.to_string()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitLnPayRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let pay = _await_pay(state, req).await?;
    Ok(Json(pay))
}
