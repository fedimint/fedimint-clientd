use anyhow::{anyhow, Error};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_core::api::InviteCode;
use fedimint_core::config::FederationId;
use multimint::MultiMint;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequest {
    pub invite_code: InviteCode,
    pub set_default: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinResponse {
    pub federation_ids: Vec<FederationId>,
}

async fn _join(mut multimint: MultiMint, req: JoinRequest) -> Result<JoinResponse, Error> {
    let _ = multimint
        .register_new(req.invite_code.clone(), req.set_default)
        .await?;

    let federation_ids = multimint.ids().await.into_iter().collect::<Vec<_>>();

    Ok(JoinResponse { federation_ids })
}

pub async fn handle_ws(state: AppState, v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<JoinRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let join = _join(state.multimint, v).await?;
    let join_json = json!(join);
    Ok(join_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<JoinRequest>,
) -> Result<Json<JoinResponse>, AppError> {
    let join = _join(state.multimint, req).await?;
    Ok(Json(join))
}
