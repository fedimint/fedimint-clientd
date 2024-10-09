use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use futures_util::StreamExt;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::secp256k1::{KeyPair, Secp256k1, SecretKey};
use multimint::fedimint_ln_client::{LightningClientModule, LnReceiveState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimExternalReceiveTweakedRequest {
    pub tweaks: Vec<u64>,
    pub private_key: SecretKey,
    pub federation_id: FederationId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimExternalReceiveTweakedResponse {
    pub status: LnReceiveState,
}

async fn _await_claim_external_receive_tweaked(
    client: ClientHandleArc,
    req: ClaimExternalReceiveTweakedRequest,
) -> Result<ClaimExternalReceiveTweakedResponse, AppError> {
    let secp = Secp256k1::new();
    let key_pair = KeyPair::from_secret_key(&secp, &req.private_key);
    let lightning_module = &client.get_first_module::<LightningClientModule>();
    let operation_id = lightning_module
        .scan_receive_for_user_tweaked(key_pair, req.tweaks, ())
        .await;

    for operation_id in operation_id {
        let mut updates = lightning_module
            .subscribe_ln_claim(operation_id)
            .await?
            .into_stream();
        info!(
            "Created claim external receive tweaked stream for operation id: {:?}",
            operation_id
        );
        while let Some(update) = updates.next().await {
            debug!("Update: {update:?}");
            match &update {
                LnReceiveState::Claimed => {
                    return Ok(ClaimExternalReceiveTweakedResponse { status: update });
                }
                LnReceiveState::Canceled { reason } => {
                    error!("Claim canceled: {}", reason);
                    return Ok(ClaimExternalReceiveTweakedResponse { status: update });
                }
                _ => {}
            }
        }
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<ClaimExternalReceiveTweakedRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let invoice = _await_claim_external_receive_tweaked(client, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ClaimExternalReceiveTweakedRequest>,
) -> Result<Json<ClaimExternalReceiveTweakedResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let invoice_response = _await_claim_external_receive_tweaked(client, req).await?;
    Ok(Json(invoice_response))
}
