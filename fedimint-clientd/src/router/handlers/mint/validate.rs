use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::Amount;
use multimint::fedimint_mint_client::{MintClientModule, OOBNotes};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub amount_msat: Amount,
}

async fn _validate(
    client: ClientHandleArc,
    req: ValidateRequest,
) -> Result<ValidateResponse, AppError> {
    let amount_msat = client
        .get_first_module::<MintClientModule>()
        .validate_notes(&req.notes)?;

    Ok(ValidateResponse { amount_msat })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<ValidateRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let client = state
        .get_client_by_prefix(&v.notes.federation_id_prefix())
        .await?;
    let validate = _validate(client, v).await?;
    let validate_json = json!(validate);
    Ok(validate_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let client = state
        .get_client_by_prefix(&req.notes.federation_id_prefix())
        .await?;
    let validate = _validate(client, req).await?;
    Ok(Json(validate))
}
