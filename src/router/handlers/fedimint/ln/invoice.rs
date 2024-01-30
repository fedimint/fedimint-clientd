use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::ClientArc;
use fedimint_core::{config::FederationId, core::OperationId, Amount};
use fedimint_ln_client::LightningClientModule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LnInvoiceRequest {
    pub amount_msat: Amount,
    pub description: String,
    pub expiry_time: Option<u64>,
    pub federation_id: Option<FederationId>,
}

#[derive(Debug, Serialize)]
pub struct LnInvoiceResponse {
    pub operation_id: OperationId,
    pub invoice: String,
}

async fn _invoice(client: ClientArc, req: LnInvoiceRequest) -> Result<LnInvoiceResponse, AppError> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let (operation_id, invoice) = lightning_module
        .create_bolt11_invoice(req.amount_msat, req.description, req.expiry_time, ())
        .await?;
    Ok(LnInvoiceResponse {
        operation_id,
        invoice: invoice.to_string(),
    })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<LnInvoiceRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let invoice = _invoice(client, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<LnInvoiceRequest>,
) -> Result<Json<LnInvoiceResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let invoice = _invoice(client, req).await?;
    Ok(Json(invoice))
}
