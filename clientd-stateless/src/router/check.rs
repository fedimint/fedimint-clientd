use axum::extract::State;
use axum::Json;
use multimint::fedimint_core::Amount;
use multimint::fedimint_mint_client::{MintClientModule, OOBNotes};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CheckRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct CheckResponse {
    pub amount_msat: Amount,
}

#[axum_macros::debug_handler]
pub async fn handle_check(
    State(state): State<AppState>,
    Json(req): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, AppError> {
    let client = state
        .get_client_by_prefix(&req.notes.federation_id_prefix())
        .await?;
    let amount_msat = client
        .get_first_module::<MintClientModule>()
        .validate_notes(req.notes)
        .await?;

    Ok(Json(CheckResponse { amount_msat }))
}
