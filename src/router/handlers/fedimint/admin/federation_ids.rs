use anyhow::Error;
use axum::{extract::State, Json};
use fedimint_core::config::FederationId;
use multimint::MultiMint;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationIdsResponse {
    pub federation_ids: Vec<FederationId>,
}

async fn _federation_ids(multimint: MultiMint) -> Result<FederationIdsResponse, Error> {
    let federation_ids = multimint.ids().await.into_iter().collect::<Vec<_>>();
    Ok(FederationIdsResponse { federation_ids })
}

pub async fn handle_ws(state: AppState, _v: Value) -> Result<Value, AppError> {
    let federation_ids = _federation_ids(state.multimint).await?;
    let federation_ids_json = json!(federation_ids);
    Ok(federation_ids_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
) -> Result<Json<FederationIdsResponse>, AppError> {
    let federation_ids = _federation_ids(state.multimint).await?;
    Ok(Json(federation_ids))
}
