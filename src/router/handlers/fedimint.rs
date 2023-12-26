use bitcoin::PublicKey;
use fedimint_ln_client::{
    LightningClientModule, LnReceiveState, OutgoingLightningPayment, PayType,
};
use itertools::Itertools;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use crate::types::fedimint::{
    AwaitInvoiceRequest, CombineRequest, CombineResponse, InfoResponse, LnInvoiceRequest,
    LnInvoiceResponse, LnPayRequest, LnPayResponse, ReissueRequest, ReissueResponse, SpendRequest,
    SpendResponse, SplitRequest, SplitResponse, SwitchGatewayRequest, ValidateRequest,
    ValidateResponse,
};

use crate::utils::{get_invoice, get_note_summary, wait_for_ln_payment};
use crate::{error::AppError, state::AppState};
use anyhow::{anyhow, Context, Result};
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
pub async fn handle_combine(req: Json<CombineRequest>) -> Result<Json<CombineResponse>, AppError> {
    let federation_id_prefix = match req
        .notes
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
        .notes
        .iter()
        .flat_map(|notes| notes.notes().iter_items().map(|(amt, note)| (amt, *note)))
        .collect();

    let combined_oob_notes = OOBNotes::new(federation_id_prefix, combined_notes);

    Ok(Json(CombineResponse {
        notes: combined_oob_notes,
    }))
}

#[axum_macros::debug_handler]
pub async fn handle_lninvoice(
    State(state): State<AppState>,
    Json(req): Json<LnInvoiceRequest>,
) -> Result<Json<LnInvoiceResponse>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    let (operation_id, invoice) = lightning_module
        .create_bolt11_invoice(req.amount_msat, req.description, req.expiry_time, ())
        .await?;
    Ok(Json(LnInvoiceResponse {
        operation_id,
        invoice: invoice.to_string(),
    }))
}

#[axum_macros::debug_handler]
pub async fn handle_awaitinvoice(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<InfoResponse>, AppError> {
    let lightning_module = &state.fm.get_first_module::<LightningClientModule>();
    let mut updates = lightning_module
        .subscribe_ln_receive(req.operation_id)
        .await?
        .into_stream();
    while let Some(update) = updates.next().await {
        match update {
            LnReceiveState::Claimed => {
                return Ok(Json(get_note_summary(&state.fm).await?));
            }
            LnReceiveState::Canceled { reason } => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!(reason),
                ))
            }
            _ => {}
        }

        info!("Update: {update:?}");
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

#[axum_macros::debug_handler]
pub async fn handle_lnpay(
    State(state): State<AppState>,
    Json(req): Json<LnPayRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let bolt11 = get_invoice(&req).await?;
    info!("Paying invoice: {bolt11}");
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.select_active_gateway().await?;

    // let gateway = lightning_module.select_active_gateway_opt().await;
    let OutgoingLightningPayment {
        payment_type,
        contract_id,
        fee,
    } = lightning_module.pay_bolt11_invoice(bolt11, ()).await?;
    let operation_id = payment_type.operation_id();
    info!("Gateway fee: {fee}, payment operation id: {operation_id}");
    if req.finish_in_background {
        wait_for_ln_payment(&state.fm, payment_type, contract_id.to_string(), true).await?;
        info!("Payment will finish in background, use await-ln-pay to get the result");
        Ok(Json(LnPayResponse {
            operation_id,
            payment_type: payment_type,
            contract_id: contract_id.to_string(),
            fee,
        }))
    } else {
        Ok(Json(
            wait_for_ln_payment(&state.fm, payment_type, contract_id.to_string(), false)
                .await?
                .context("expected a response")?,
        ))
    }
}

#[axum_macros::debug_handler]
pub async fn handle_awaitlnpay(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<LnPayResponse>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    let ln_pay_details = lightning_module
        .get_ln_pay_details_for(req.operation_id)
        .await?;
    let payment_type = if ln_pay_details.is_internal_payment {
        PayType::Internal(req.operation_id)
    } else {
        PayType::Lightning(req.operation_id)
    };
    Ok(Json(
        wait_for_ln_payment(
            &state.fm,
            payment_type,
            ln_pay_details.contract_id.to_string(),
            false,
        )
        .await?
        .context("expected a response")?,
    ))
}

#[axum_macros::debug_handler]
pub async fn handle_listgateways(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    let gateways = lightning_module.fetch_registered_gateways().await?;
    if gateways.is_empty() {
        return Ok(Json(serde_json::to_value(Vec::<String>::new()).unwrap()));
    }

    let mut gateways_json = json!(&gateways);
    let active_gateway = lightning_module.select_active_gateway().await?;

    gateways_json
        .as_array_mut()
        .expect("gateways_json is not an array")
        .iter_mut()
        .for_each(|gateway| {
            if gateway["node_pub_key"] == json!(active_gateway.node_pub_key) {
                gateway["active"] = json!(true);
            } else {
                gateway["active"] = json!(false);
            }
        });
    Ok(Json(serde_json::to_value(gateways_json).unwrap()))
}

#[axum_macros::debug_handler]
pub async fn handle_switchgateway(
    State(state): State<AppState>,
    Json(req): Json<SwitchGatewayRequest>,
) -> Result<Json<Value>, AppError> {
    let public_key = PublicKey::from_str(&req.gateway_id)?;
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    lightning_module.set_active_gateway(&public_key).await?;
    let gateway = lightning_module.select_active_gateway().await?;
    let mut gateway_json = json!(&gateway);
    gateway_json["active"] = json!(true);
    Ok(Json(serde_json::to_value(gateway_json).unwrap()))
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

// #[axum_macros::debug_handler]
// pub async fn handle_printsecret() -> Result<(), AppError> {
//     // TODO: Implement this function
//     Ok(())
// }

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
