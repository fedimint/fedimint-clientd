use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::cashu::Keyset;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub struct KeysetsResponse {
    keysets: Vec<Keyset>,
}

#[axum_macros::debug_handler]
pub async fn handle_keysets(
    State(state): State<AppState>,
) -> Result<Json<KeysetsResponse>, AppError> {
    let mut keysets = Vec::<Keyset>::new();
    let ids = state.multimint.ids().await;
    for id in ids {
        keysets.push(Keyset::from(id))
    }

    Ok(Json(KeysetsResponse { keysets }))
}
