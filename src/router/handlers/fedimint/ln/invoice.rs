use axum::{extract::State, Json};
use fedimint_core::{core::OperationId, Amount};
use fedimint_ln_client::LightningClientModule;
use serde::{Deserialize, Serialize};

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

#[axum_macros::debug_handler]
pub async fn handle_invoice(
    State(state): State<AppState>,
    Json(req): Json<LnInvoiceRequest>,
) -> Result<Json<LnInvoiceResponse>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let (operation_id, invoice) = lightning_module
        .create_bolt11_invoice(req.amount_msat, req.description, req.expiry_time, ())
        .await?;
    Ok(Json(LnInvoiceResponse {
        operation_id,
        invoice: invoice.to_string(),
    }))
}
