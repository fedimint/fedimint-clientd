use axum::{extract::ws::Message, extract::State, Json};
use fedimint_core::Amount;
use fedimint_mint_client::{MintClientModule, OOBNotes};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub amount_msat: Amount,
}

async fn _validate(state: AppState, req: ValidateRequest) -> Result<ValidateResponse, AppError> {
    let amount_msat =
        state
            .fm
            .get_first_module::<MintClientModule>()
            .validate_notes(req.notes)
            .await?;

    Ok(ValidateResponse { amount_msat })
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Message, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let validate = _validate(state, v).await?;
    let validate_json = json!(validate);
    Ok(Message::Text(validate_json.to_string()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let validate = _validate(state, req).await?;
    Ok(Json(validate))
}
