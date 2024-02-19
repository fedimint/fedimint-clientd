use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_core::config::FederationId;
use fedimint_core::Amount;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SwapRequest {
    pub notes: OOBNotes,
    pub federation_id: Option<FederationId>,
}

#[derive(Debug, Serialize)]
pub struct SwapResponse {
    pub amount_msat: Amount,
}

#[axum_macros::debug_handler]
pub async fn handle_swap(
    State(state): State<AppState>,
    Json(req): Json<SwapRequest>,
) -> Result<Json<SwapResponse>, AppError> {
    let amount_msat = req.notes.total_amount();

    let client = state.get_client(req.federation_id).await?;
    let mint = client.get_first_module::<MintClientModule>();

    let operation_id = mint.reissue_external_notes(req.notes, ()).await?;
    let mut updates = mint
        .subscribe_reissue_external_notes(operation_id)
        .await
        .unwrap()
        .into_stream();

    while let Some(update) = updates.next().await {
        let update_clone = update.clone();
        if let fedimint_mint_client::ReissueExternalNotesState::Failed(e) = update {
            Err(AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!(e)))?;
        }

        info!("Update: {update_clone:?}");
    }

    Ok(Json(SwapResponse { amount_msat }))
}
