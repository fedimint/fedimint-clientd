use axum::{extract::State, Json};
use fedimint_ln_client::LightningClientModule;

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{LnInvoiceRequest, LnInvoiceResponse},
};

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
