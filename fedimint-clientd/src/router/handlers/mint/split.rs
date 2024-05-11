use std::collections::BTreeMap;

use axum::http::StatusCode;
use axum::Json;
use multimint::fedimint_core::{Amount, TieredMulti};
use multimint::fedimint_mint_client::OOBNotes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitResponse {
    pub notes: BTreeMap<Amount, OOBNotes>,
}

async fn _split(req: SplitRequest) -> Result<SplitResponse, AppError> {
    let federation = req.notes.federation_id_prefix();
    let notes = req
        .notes
        .notes()
        .iter()
        .map(|(amount, notes)| {
            let notes = notes
                .iter()
                .map(|note| {
                    OOBNotes::new(
                        federation,
                        TieredMulti::new(vec![(*amount, vec![*note])].into_iter().collect()),
                    )
                })
                .collect::<Vec<_>>();
            (*amount, notes[0].clone()) // clone the amount and return a single
                                        // OOBNotes
        })
        .collect::<BTreeMap<_, _>>();

    Ok(SplitResponse { notes })
}

pub async fn handle_ws(v: Value) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!("Invalid request: {}", e),
        )
    })?;
    let split = _split(v).await?;
    let split_json = json!(split);
    Ok(split_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(Json(req): Json<SplitRequest>) -> Result<Json<SplitResponse>, AppError> {
    let split = _split(req).await?;
    Ok(Json(split))
}
