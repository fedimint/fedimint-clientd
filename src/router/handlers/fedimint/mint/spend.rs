use std::time::Duration;

use axum::{extract::State, Json};
use fedimint_core::core::OperationId;
use fedimint_core::Amount;
use fedimint_mint_client::OOBNotes;
use fedimint_mint_client::{MintClientModule, SelectNotesWithAtleastAmount};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct SpendRequest {
    pub amount_msat: Amount,
    pub allow_overpay: bool,
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct SpendResponse {
    pub operation: OperationId,
    pub notes: OOBNotes,
}

#[axum_macros::debug_handler]
pub async fn handle_spend(
    State(state): State<AppState>,
    Json(req): Json<SpendRequest>,
) -> Result<Json<SpendResponse>, AppError> {
    warn!("The client will try to double-spend these notes after the duration specified by the --timeout option to recover any unclaimed e-cash.");

    let mint_module = state.fm.get_first_module::<MintClientModule>();
    let timeout = Duration::from_secs(req.timeout);
    // let (operation, notes) = if req.allow_overpay {  TODO: not backported yet
    let (operation, notes) = mint_module
        .spend_notes_with_selector(&SelectNotesWithAtleastAmount, req.amount_msat, timeout, ())
        .await?;

    let overspend_amount = notes.total_amount() - req.amount_msat;
    if overspend_amount != Amount::ZERO {
        warn!(
            "Selected notes {} worth more than requested",
            overspend_amount
        );
    }
    info!("Spend e-cash operation: {operation}");
    Ok(Json(SpendResponse { operation, notes }))
    // } else {
    // mint_module
    //     .spend_notes_with_selector(&SelectNotesWithExactAmount, req.amount, timeout, ()) TODO: not backported yet
    //     .await?
    // };
}
