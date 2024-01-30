use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::ClientArc;
use fedimint_core::core::OperationId;
use fedimint_ln_client::{LightningClientModule, LnReceiveState};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;
use fedimint_core::config::FederationId;
use futures_util::StreamExt;

use crate::{
    error::AppError,
    router::handlers::fedimint::admin::{get_note_summary, info::InfoResponse},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AwaitInvoiceRequest {
    pub operation_id: OperationId,
    pub federation_id: Option<FederationId>,
}

async fn _await_invoice(
    client: ClientArc,
    req: AwaitInvoiceRequest,
) -> Result<InfoResponse, AppError> {
    let lightning_module = &client.get_first_module::<LightningClientModule>();
    let mut updates = lightning_module
        .subscribe_ln_receive(req.operation_id)
        .await?
        .into_stream();
    while let Some(update) = updates.next().await {
        match update {
            LnReceiveState::Claimed => {
                return Ok(get_note_summary(&client).await?);
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
    let v = serde_json::from_value::<AwaitInvoiceRequest>(v).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!("Invalid request: {}", e),
        )
    })?;
    let client = state.get_client(v.federation_id).await?;
    let invoice = _await_invoice(client, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<InfoResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let invoice = _await_invoice(client, req).await?;
    Ok(Json(invoice))
}
