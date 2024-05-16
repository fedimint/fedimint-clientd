use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use futures_util::StreamExt;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::Amount;
use multimint::fedimint_mint_client::{MintClientModule, OOBNotes, ReissueExternalNotesState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueResponse {
    pub amount_msat: Amount,
}

async fn _reissue(
    client: ClientHandleArc,
    req: ReissueRequest,
) -> Result<ReissueResponse, AppError> {
    let amount_msat = req.notes.total_amount();

    let mint = client.get_first_module::<MintClientModule>();

    let operation_id = mint.reissue_external_notes(req.notes, ()).await?;
    let mut updates = mint
        .subscribe_reissue_external_notes(operation_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to subscribe to reissue operation: {}", e))?
        .into_stream();

    while let Some(update) = updates.next().await {
        let update_clone = update.clone();
        if let ReissueExternalNotesState::Failed(e) = update {
            Err(AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!(e)))?;
        }

        info!("Update: {update_clone:?}");
    }

    Ok(ReissueResponse { amount_msat })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<ReissueRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state
        .get_client_by_prefix(&v.notes.federation_id_prefix())
        .await?;
    let reissue = _reissue(client, v).await?;
    let reissue_json = json!(reissue);
    Ok(reissue_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    let client = state
        .get_client_by_prefix(&req.notes.federation_id_prefix())
        .await?;
    let reissue = _reissue(client, req).await?;
    Ok(Json(reissue))
}
