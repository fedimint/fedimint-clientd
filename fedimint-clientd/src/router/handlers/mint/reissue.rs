use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use futures_util::StreamExt;
use multimint::cdk::amount::SplitTarget;
use multimint::cdk::nuts::Token;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::Amount;
use multimint::fedimint_mint_client::{MintClientModule, OOBNotes, ReissueExternalNotesState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::error::AppError;
use crate::state::AppState;
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReissueRequest {
    pub ecash: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueResponse {
    pub amount_msat: Amount,
}

async fn _reissue(client: ClientHandleArc, notes: OOBNotes) -> Result<ReissueResponse, AppError> {
    let amount_msat = notes.total_amount();

    let mint = client.get_first_module::<MintClientModule>();

    let operation_id = mint.reissue_external_notes(notes, ()).await?;
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
    let ecash = v.ecash.clone();
    if let Ok(notes) = serde_json::from_str::<OOBNotes>(&ecash) {
        let client = state
            .get_client_by_prefix(&notes.federation_id_prefix())
            .await?;
        let reissue = _reissue(client, notes).await?;
        let reissue_json = json!(reissue);
        Ok(reissue_json)
    } else if let Ok(token) = Token::from_str(&ecash) {
        let amount = state
            .multimint
            .cashu_wallet
            .lock()
            .await
            .receive(&token.to_string(), &SplitTarget::None, None)
            .await?;
        Ok(json!(ReissueResponse {
            amount_msat: Amount::from_sats(u64::from(amount)),
        }))
    } else {
        Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!("Invalid ecash format"),
        ))
    }
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    match Token::from_str(&req.ecash) {
        Ok(token) => {
            let amount = state
                .multimint
                .cashu_wallet
                .lock()
                .await
                .receive(&token.to_string(), &SplitTarget::None, None)
                .await?;
            Ok(Json(ReissueResponse {
                amount_msat: Amount::from_sats(u64::from(amount)),
            }))
        }
        Err(_) => match serde_json::from_str::<OOBNotes>(&req.ecash) {
            Ok(notes) => {
                let client = state
                    .get_client_by_prefix(&notes.federation_id_prefix())
                    .await?;
                let reissue = _reissue(client, notes).await?;
                Ok(Json(reissue))
            }
            Err(e) => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("Invalid ecash format: {}", e),
            )),
        },
    }
}
