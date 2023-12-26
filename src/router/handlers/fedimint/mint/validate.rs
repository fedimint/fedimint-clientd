use axum::{extract::State, Json};
use fedimint_core::Amount;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub amount_msat: Amount,
}

#[axum_macros::debug_handler]
pub async fn handle_validate(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let amount_msat =
        state
            .fm
            .get_first_module::<MintClientModule>()
            .validate_notes(req.notes)
            .await?;

    Ok(Json(ValidateResponse { amount_msat }))
}
