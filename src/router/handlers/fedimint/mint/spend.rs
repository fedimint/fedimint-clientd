use std::time::Duration;

use axum::{extract::State, Json};
use fedimint_core::core::OperationId;
use fedimint_core::Amount;
use fedimint_mint_client::OOBNotes;
use fedimint_mint_client::{MintClientModule, SelectNotesWithAtleastAmount};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

async fn _spend(state: AppState, req: SpendRequest) -> Result<SpendResponse, AppError> {
    warn!("The client will try to double-spend these notes after the duration specified by the --timeout option to recover any unclaimed e-cash.");

    let mint_module = state.fm.get_first_module::<MintClientModule>();
    let timeout = Duration::from_secs(req.timeout);
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
    Ok(SpendResponse { operation, notes })
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let spend = _spend(state, v).await?;
    let spend_json = json!(spend);
    Ok(spend_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<SpendRequest>,
) -> Result<Json<SpendResponse>, AppError> {
    let spend = _spend(state, req).await?;
    Ok(Json(spend))
}
