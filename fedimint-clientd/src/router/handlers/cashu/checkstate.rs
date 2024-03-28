use std::str::FromStr;

use axum::extract::State;
use axum::Json;
use fedimint_core::config::FederationIdPrefix;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use serde::{Deserialize, Serialize};

use super::CashuToken;
use crate::error::AppError;
use crate::fedimint::mint::OOBNotesJson;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CheckStateRequest {
    pub token: CashuToken,
}

#[derive(Debug, Serialize)]
enum TokenState {
    UNSPENT,
    PENDING,
    SPENT,
}

#[derive(Debug, Serialize)]
struct States {
    y: String,
    state: TokenState,
    witness: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckStateResponse {
    pub amount_sat: u64,
}

#[axum_macros::debug_handler]
pub async fn handle_checkstate(
    State(state): State<AppState>,
    Json(req): Json<CheckStateRequest>,
) -> Result<Json<CheckStateResponse>, AppError> {
    let client = state.get_cashu_client().await?;
    let federation_id = client.federation_id();
    let notes = OOBNotesJson::from_str(&req.token.token[0].mint)?;
    let oob_notes = OOBNotes::new(
        FederationIdPrefix::from_str(&notes.federation_id_prefix)?,
        notes.notes,
    );
    let amount_sat = client
        .get_first_module::<MintClientModule>()
        .validate_notes(oob_notes)
        .await?
        .try_into_sats()
        .unwrap();

    Ok(Json(CheckStateResponse { amount_sat }))
}
