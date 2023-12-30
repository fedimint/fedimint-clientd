use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_core::core::OperationId;
use fedimint_ln_client::{LightningClientModule, LnReceiveState};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use crate::{
    error::AppError,
    router::handlers::fedimint::admin::{get_note_summary, info::InfoResponse},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AwaitInvoiceRequest {
    pub operation_id: OperationId,
}

async fn _await_invoice(
    state: AppState,
    req: AwaitInvoiceRequest,
) -> Result<InfoResponse, AppError> {
    let lightning_module = &state.fm.get_first_module::<LightningClientModule>();
    let mut updates = lightning_module
        .subscribe_ln_receive(req.operation_id)
        .await?
        .into_stream();
    while let Some(update) = updates.next().await {
        match update {
            LnReceiveState::Claimed => {
                return Ok(get_note_summary(&state.fm).await?);
            }
            LnReceiveState::Canceled { reason } => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!(reason),
                ))
            }
            _ => {}
        }

        info!("Update: {update:?}");
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let invoice = _await_invoice(state, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<InfoResponse>, AppError> {
    let invoice = _await_invoice(state, req).await?;
    Ok(Json(invoice))
}
