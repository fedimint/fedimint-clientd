use std::collections::BTreeMap;
use std::time::Duration;

use crate::types::fedimint::*;
use crate::{error::AppError, state::AppState};
use anyhow::{anyhow, Result};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use fedimint_core::{Amount, TieredMulti};
use fedimint_mint_client::{
    MintClientModule,
    OOBNotes, // SelectNotesWithExactAmount, TODO: not backported yet
    SelectNotesWithAtleastAmount,
};
use fedimint_wallet_client::WalletClientModule;
use futures::StreamExt;
use tracing::{info, warn};

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

#[axum_macros::debug_handler]
pub async fn handle_spend(
    State(state): State<AppState>,
    Json(req): Json<SpendRequest>,
) -> Result<Json<SpendResponse>, AppError> {
    warn!("The client will try to double-spend these notes after the duration specified by the --timeout option to recover any unclaimed e-cash.");

    let mint_module = state.fm.get_first_module::<MintClientModule>();
    let timeout = Duration::from_secs(req.timeout);
    // let (operation, notes) = if req.allow_overpay {  TODO: not backported yet
    let (operation, notes) = mint_module
        .spend_notes_with_selector(&SelectNotesWithAtleastAmount, req.amount_msat, timeout, ())
        .await?;

    let overspend_amount = notes.total_amount() - req.amount_msat;
    if overspend_amount != Amount::ZERO {
        warn!(
            "Selected notes {} worth more than requested",
            overspend_amount
        );
    }
    info!("Spend e-cash operation: {operation}");
    Ok(Json(SpendResponse { operation, notes }))
    // } else {
    // mint_module
    //     .spend_notes_with_selector(&SelectNotesWithExactAmount, req.amount, timeout, ()) TODO: not backported yet
    //     .await?
    // };
}

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

#[axum_macros::debug_handler]
pub async fn handle_split(Json(req): Json<SplitRequest>) -> Result<Json<SplitResponse>, AppError> {
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
            (*amount, notes[0].clone()) // clone the amount and return a single OOBNotes
        })
        .collect::<BTreeMap<_, _>>();

    Ok(Json(SplitResponse { notes }))
}

#[axum_macros::debug_handler]
pub async fn handle_combine() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_lninvoice() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_awaitinvoice() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_lnpay() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_awaitlnpay() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_listgateways() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_switchgateway() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_depositaddress() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_awaitdeposit() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_withdraw() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_backup() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_discoverversion() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_restore() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_printsecret() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_listoperations() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_module() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_config() -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}
