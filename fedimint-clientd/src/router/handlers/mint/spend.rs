use std::time::Duration;

use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::core::OperationId;
use multimint::fedimint_core::Amount;
use multimint::fedimint_mint_client::{
    MintClientModule, OOBNotes, SelectNotesWithAtleastAmount, SelectNotesWithExactAmount,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendRequest {
    pub amount_msat: Amount,
    pub allow_overpay: bool,
    pub timeout: u64,
    pub include_invite: bool,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendResponse {
    pub operation: OperationId,
    pub notes: OOBNotes,
}

async fn _spend(client: ClientHandleArc, req: SpendRequest) -> Result<SpendResponse, AppError> {
    warn!("The client will try to double-spend these notes after the duration specified by the --timeout option to recover any unclaimed e-cash.");
    let mint_module = client.get_first_module::<MintClientModule>();
    let timeout = Duration::from_secs(req.timeout);
    let (operation, notes) = if req.allow_overpay {
        let (operation, notes) = mint_module
            .spend_notes_with_selector(
                &SelectNotesWithAtleastAmount,
                req.amount_msat,
                timeout,
                req.include_invite,
                (),
            )
            .await?;

        let overspend_amount = notes.total_amount() - req.amount_msat;
        if overspend_amount != Amount::ZERO {
            warn!(
                "Selected notes {} worth more than requested",
                overspend_amount
            );
        }

        (operation, notes)
    } else {
        mint_module
            .spend_notes_with_selector(
                &SelectNotesWithExactAmount,
                req.amount_msat,
                timeout,
                req.include_invite,
                (),
            )
            .await?
    };
    info!("Spend e-cash operation: {:?}", operation);
    Ok(SpendResponse { operation, notes })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<SpendRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let spend = _spend(client, v).await?;
    let spend_json = json!(spend);
    Ok(spend_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<SpendRequest>,
) -> Result<Json<SpendResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let spend = _spend(client, req).await?;
    Ok(Json(spend))
}
