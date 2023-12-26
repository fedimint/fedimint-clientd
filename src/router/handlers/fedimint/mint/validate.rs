use axum::{extract::State, Json};
use fedimint_mint_client::MintClientModule;

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{ValidateRequest, ValidateResponse},
};

#[axum_macros::debug_handler]
pub async fn handle_validate(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, AppError> {
    let amount_msat =
        state
            .fm
            .get_first_module::<MintClientModule>()
            .validate_notes(req.notes)
            .await?;

    Ok(Json(ValidateResponse { amount_msat }))
}
