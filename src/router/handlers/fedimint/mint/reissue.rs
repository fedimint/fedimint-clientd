use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{ReissueRequest, ReissueResponse},
};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_mint_client::MintClientModule;
use futures::StreamExt;
use tracing::info;

#[axum_macros::debug_handler]
pub async fn handle_reissue(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    let amount_msat = req.notes.total_amount();

    let mint = state.fm.get_first_module::<MintClientModule>();

    let operation_id = mint.reissue_external_notes(req.notes, ()).await?;
    let mut updates = mint
        .subscribe_reissue_external_notes(operation_id)
        .await
        .unwrap()
        .into_stream();

    while let Some(update) = updates.next().await {
        let update_clone = update.clone();
        if let fedimint_mint_client::ReissueExternalNotesState::Failed(e) = update {
            Err(AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!(e)))?;
        }

        info!("Update: {update_clone:?}");
    }

    Ok(Json(ReissueResponse { amount_msat }))
}
