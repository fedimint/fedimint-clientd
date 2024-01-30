use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode::BAD_REQUEST, Json};
use fedimint_core::Amount;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ReissueRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ReissueResponse {
    pub amount_msat: Amount,
}

async fn _reissue(state: AppState, req: ReissueRequest) -> Result<ReissueResponse, AppError> {
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

    Ok(ReissueResponse { amount_msat })
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let reissue = _reissue(state, v).await?;
    let reissue_json = json!(reissue);
    Ok(reissue_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    let reissue = _reissue(state, req).await?;
    Ok(Json(reissue))
}
