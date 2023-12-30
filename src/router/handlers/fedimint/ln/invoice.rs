use axum::{extract::State, Json};
use fedimint_core::{core::OperationId, Amount};
use fedimint_ln_client::LightningClientModule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LnInvoiceRequest {
    pub amount_msat: Amount,
    pub description: String,
    pub expiry_time: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct LnInvoiceResponse {
    pub operation_id: OperationId,
    pub invoice: String,
}

async fn _invoice(state: AppState, req: LnInvoiceRequest) -> Result<LnInvoiceResponse, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let (operation_id, invoice) = lightning_module
        .create_bolt11_invoice(req.amount_msat, req.description, req.expiry_time, ())
        .await?;
    Ok(LnInvoiceResponse {
        operation_id,
        invoice: invoice.to_string(),
    })
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let invoice = _invoice(state, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<LnInvoiceRequest>,
) -> Result<Json<LnInvoiceResponse>, AppError> {
    let invoice = _invoice(state, req).await?;
    Ok(Json(invoice))
}
