use anyhow::anyhow;
use axum::http::StatusCode;
use axum::Json;
use fedimint_mint_client::OOBNotes;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineRequest {
    pub notes_vec: Vec<OOBNotes>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineResponse {
    pub notes: OOBNotes,
}

async fn _combine(req: CombineRequest) -> Result<CombineResponse, AppError> {
    let federation_id_prefix = match req
        .notes_vec
        .iter()
        .map(|notes| notes.federation_id_prefix())
        .all_equal_value()
    {
        Ok(id) => id,
        Err(None) => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!("E-cash notes strings from different federations"),
        ))?,
        Err(Some((a, b))) => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!(
                "E-cash notes strings from different federations: {:?} and {:?}",
                a,
                b
            ),
        ))?,
    };

    let combined_notes = req
        .notes_vec
        .iter()
        .flat_map(|notes| notes.notes().iter_items().map(|(amt, note)| (amt, *note)))
        .collect();

    let combined_oob_notes = OOBNotes::new(federation_id_prefix, combined_notes);

    Ok(CombineResponse {
        notes: combined_oob_notes,
    })
}

pub async fn handle_ws(v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value(v)
        .map_err(|e| AppError::new(StatusCode::BAD_REQUEST, anyhow!("Invalid request: {}", e)))?;
    let combine = _combine(v).await?;
    let combine_json = json!(combine);
    Ok(combine_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    Json(req): Json<CombineRequest>,
) -> Result<Json<CombineResponse>, AppError> {
    let combine = _combine(req).await?;
    Ok(Json(combine))
}
