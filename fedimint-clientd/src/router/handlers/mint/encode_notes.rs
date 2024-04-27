use std::str::FromStr;

use anyhow::anyhow;
use axum::http::StatusCode;
use axum::Json;
use fedimint_core::config::FederationIdPrefix;
use fedimint_mint_client::OOBNotes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::OOBNotesJson;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeRequest {
    pub notes_json_str: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeResponse {
    pub notes: OOBNotes,
}

async fn _encode_notes(req: EncodeRequest) -> Result<EncodeResponse, AppError> {
    let notes = serde_json::from_str::<OOBNotesJson>(&req.notes_json_str)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid notes: {}", e)))?;
    let prefix = FederationIdPrefix::from_str(&notes.federation_id_prefix)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid prefix: {}", e)))?;
    let notes = OOBNotes::new(prefix, notes.notes);

    Ok(EncodeResponse { notes })
}

pub async fn handle_ws(v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<EncodeRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let encode = _encode_notes(v).await?;
    let encode_json = json!(encode);
    Ok(encode_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(Json(req): Json<EncodeRequest>) -> Result<Json<EncodeResponse>, AppError> {
    let decode = _encode_notes(req).await?;
    Ok(Json(decode))
}
