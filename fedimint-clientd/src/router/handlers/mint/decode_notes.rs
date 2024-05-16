use anyhow::anyhow;
use axum::http::StatusCode;
use axum::Json;
use multimint::fedimint_mint_client::OOBNotes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeResponse {
    pub notes_json: Value,
}

async fn _decode_notes(req: DecodeRequest) -> Result<DecodeResponse, AppError> {
    let notes_json = req
        .notes
        .notes_json()
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid notes: {}", e)))?;

    Ok(DecodeResponse { notes_json })
}

pub async fn handle_ws(v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value::<DecodeRequest>(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let decode = _decode_notes(v).await?;
    let decode_json = json!(decode);
    Ok(decode_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(Json(req): Json<DecodeRequest>) -> Result<Json<DecodeResponse>, AppError> {
    let decode = _decode_notes(req).await?;
    Ok(Json(decode))
}
