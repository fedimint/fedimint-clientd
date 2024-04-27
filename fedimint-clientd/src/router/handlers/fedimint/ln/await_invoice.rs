use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_client::ClientHandleArc;
use fedimint_core::config::FederationId;
use fedimint_core::core::OperationId;
use fedimint_ln_client::{LightningClientModule, LnReceiveState};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceRequest {
    pub operation_id: OperationId,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceResponse {
    pub status: LnReceiveState,
}

async fn _await_invoice(
    client: ClientHandleArc,
    req: AwaitInvoiceRequest,
) -> Result<AwaitInvoiceResponse, AppError> {
    let lightning_module = &client.get_first_module::<LightningClientModule>();
    let mut updates = lightning_module
        .subscribe_ln_receive(req.operation_id)
        .await?
        .into_stream();
    info!(
        "Created await invoice stream for operation id: {}",
        req.operation_id
    );
    while let Some(update) = updates.next().await {
        debug!("Update: {update:?}");
        match &update {
            LnReceiveState::Claimed => {
                return Ok(AwaitInvoiceResponse { status: update });
            }
            LnReceiveState::Canceled { reason } => {
                error!("Invoice canceled: {}", reason);
                return Ok(AwaitInvoiceResponse { status: update });
            }
            _ => {}
        }
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<AwaitInvoiceRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let invoice_response = _await_invoice(client, v).await?;
    Ok(json!(invoice_response))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<AwaitInvoiceResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let invoice_response = _await_invoice(client, req).await?;
    Ok(Json(invoice_response))
}
