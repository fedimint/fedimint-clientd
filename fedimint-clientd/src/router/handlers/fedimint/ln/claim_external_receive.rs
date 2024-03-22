use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::util::key::KeyPair;
use fedimint_client::ClientHandleArc;
use fedimint_core::config::FederationId;
use fedimint_ln_client::{LightningClientModule, LnReceiveState};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use crate::error::AppError;
use crate::router::handlers::fedimint::admin::get_note_summary;
use crate::router::handlers::fedimint::admin::info::InfoResponse;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimExternalReceiveRequest {
    pub private_key: SecretKey,
    pub federation_id: FederationId,
}

async fn _await_claim_external_receive(
    client: ClientHandleArc,
    req: ClaimExternalReceiveRequest,
) -> Result<InfoResponse, AppError> {
    let secp = Secp256k1::new();
    let key_pair = KeyPair::from_secret_key(&secp, &req.private_key);
    let lightning_module = &client.get_first_module::<LightningClientModule>();
    let operation_id = lightning_module.scan_receive_for_user(key_pair, ()).await?;
    let mut updates = lightning_module
        .subscribe_ln_claim(operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        info!("Update: {update:?}");
        match update {
            LnReceiveState::Claimed => {
                return Ok(get_note_summary(&client).await?);
            }
            LnReceiveState::Canceled { reason } => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!(reason),
                ))
            }
            _ => {}
        }

        info!("Update: {update:?}");
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<ClaimExternalReceiveRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state.get_client(v.federation_id).await?;
    let invoice = _await_claim_external_receive(client, v).await?;
    let invoice_json = json!(invoice);
    Ok(invoice_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ClaimExternalReceiveRequest>,
) -> Result<Json<InfoResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let invoice = _await_claim_external_receive(client, req).await?;
    Ok(Json(invoice))
}
