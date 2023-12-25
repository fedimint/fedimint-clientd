use std::time::Duration;

use crate::types::fedimint::{
    InfoResponse, ReissueRequest, ReissueResponse, SpendRequest, SpendResponse,
};
use crate::{error::AppError, state::AppState};
use anyhow::{anyhow, Result};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use fedimint_core::config::FederationId;
use fedimint_core::Amount;
use fedimint_mint_client::{
    MintClientModule, SelectNotesWithAtleastAmount, SelectNotesWithExactAmount,
};
use fedimint_wallet_client::WalletClientModule;
use futures::StreamExt;
use tracing::{info, warn};

#[axum_macros::debug_handler]
pub async fn handle_federation_id(
    State(state): State<AppState>,
) -> Result<Json<FederationId>, AppError> {
    Ok(Json(state.fm.federation_id()))
}

#[axum_macros::debug_handler]
pub async fn handle_info(State(state): State<AppState>) -> Result<Json<InfoResponse>, AppError> {
    let mint_client = state.fm.get_first_module::<MintClientModule>();
    let wallet_client = state.fm.get_first_module::<WalletClientModule>();
    let summary = mint_client
        .get_wallet_summary(
            &mut state
                .fm
                .db()
                .begin_transaction_nc()
                .await
                .to_ref_with_prefix_module_id(1),
        )
        .await;
    Ok(Json(InfoResponse {
        federation_id: state.fm.federation_id(),
        network: wallet_client.get_network().to_string(),
        meta: state.fm.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    }))
}

#[axum_macros::debug_handler]
pub async fn handle_reissue(
    State(state): State<AppState>,
    Json(req): Json<ReissueRequest>,
) -> Result<Json<ReissueResponse>, AppError> {
    let amount = req.notes.total_amount();

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

    Ok(Json(ReissueResponse { amount }))
}

#[axum_macros::debug_handler]
pub async fn handle_spend(
    State(state): State<AppState>,
    Json(req): Json<SpendRequest>,
) -> Result<Json<SpendResponse>, AppError> {
    warn!("The client will try to double-spend these notes after the duration specified by the --timeout option to recover any unclaimed e-cash.");

    let mint_module = state.fm.get_first_module::<MintClientModule>();
    let timeout = Duration::from_secs(req.timeout);
    let (operation, notes) = if req.allow_overpay {
        let (operation, notes) = mint_module
            .spend_notes_with_selector(&SelectNotesWithAtleastAmount, req.amount, timeout, ())
            .await?;

        let overspend_amount = notes.total_amount() - req.amount;
        if overspend_amount != Amount::ZERO {
            warn!(
                "Selected notes {} worth more than requested",
                overspend_amount
            );
        }

        (operation, notes)
    } else {
        mint_module
            .spend_notes_with_selector(&SelectNotesWithExactAmount, req.amount, timeout, ())
            .await?
    };
    info!("Spend e-cash operation: {operation}");

    Ok(Json(SpendResponse { operation, notes }))
}
