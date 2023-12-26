use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_core::Amount;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ReissueRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ReissueResponse {
    pub amount_msat: Amount,
}

#[axum_macros::debug_handler]
pub async fn handle_reissue(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    let amount_msat = req.notes.total_amount();

    let mint = state.fm.get_first_module::<MintClientModule>();

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

    Ok(Json(ReissueResponse { amount_msat }))
}
