use bitcoin::secp256k1::PublicKey;
use fedimint_client::backup::Metadata;
use fedimint_core::time::now;
use fedimint_ln_client::{
    LightningClientModule, LnReceiveState, OutgoingLightningPayment, PayType,
};
use itertools::Itertools;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};
use time::format_description::well_known::iso8601;
use time::OffsetDateTime;

use crate::types::fedimint::{
    AwaitDepositRequest, AwaitDepositResponse, AwaitInvoiceRequest, BackupRequest, CombineRequest,
    CombineResponse, DepositAddressRequest, DepositAddressResponse, InfoResponse,
    ListOperationsRequest, LnInvoiceRequest, LnInvoiceResponse, LnPayRequest, LnPayResponse,
    OperationOutput, ReissueRequest, ReissueResponse, SpendRequest, SpendResponse, SplitRequest,
    SplitResponse, SwitchGatewayRequest, ValidateRequest, ValidateResponse, WithdrawRequest,
    WithdrawResponse,
};

use crate::utils::{get_invoice, get_note_summary, wait_for_ln_payment};
use crate::{error::AppError, state::AppState};
use anyhow::{anyhow, Context, Result};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use bitcoin_hashes::hex::ToHex;
use fedimint_core::{Amount, BitcoinAmountOrAll, TieredMulti};
use fedimint_mint_client::{
    MintClientModule,
    OOBNotes, // SelectNotesWithExactAmount, TODO: not backported yet
    SelectNotesWithAtleastAmount,
};
use fedimint_wallet_client::{DepositState, WalletClientModule, WithdrawState};
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
pub async fn handle_depositaddress(
    State(state): State<AppState>,
    Json(req): Json<DepositAddressRequest>,
) -> Result<Json<DepositAddressResponse>, AppError> {
    let wallet_client = state.fm.get_first_module::<WalletClientModule>();
    let (operation_id, address) = wallet_client
        .get_deposit_address(now() + Duration::from_secs(req.timeout), ())
        .await?;
    Ok(Json(DepositAddressResponse {
        operation_id,
        address,
    }))
}

#[axum_macros::debug_handler]
pub async fn handle_awaitdeposit(
    State(state): State<AppState>,
    Json(req): Json<AwaitDepositRequest>,
) -> Result<Json<AwaitDepositResponse>, AppError> {
    let mut updates = state
        .fm
        .get_first_module::<WalletClientModule>()
        .subscribe_deposit_updates(req.operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        match update {
            DepositState::Confirmed(tx) => {
                return Ok(Json(AwaitDepositResponse {
                    status: DepositState::Confirmed(tx),
                }))
            }
            DepositState::Claimed(tx) => {
                return Ok(Json(AwaitDepositResponse {
                    status: DepositState::Claimed(tx),
                }))
            }
            DepositState::Failed(reason) => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!(reason),
                ))
            }
            _ => {}
        }
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}

#[axum_macros::debug_handler]
pub async fn handle_withdraw(
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, AppError> {
    let wallet_module = state.fm.get_first_module::<WalletClientModule>();
    let (amount, fees) = match req.amount_msat {
        // If the amount is "all", then we need to subtract the fees from
        // the amount we are withdrawing
        BitcoinAmountOrAll::All => {
            let balance = bitcoin::Amount::from_sat(state.fm.get_balance().await.msats / 1000);
            let fees = wallet_module
                .get_withdraw_fees(req.address.clone(), balance)
                .await?;
            let amount = balance.checked_sub(fees.amount());
            if amount.is_none() {
                Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    anyhow!("Insufficient balance to pay fees"),
                ))?;
            }
            (amount.unwrap(), fees)
        }
        BitcoinAmountOrAll::Amount(amount) => (
            amount,
            wallet_module
                .get_withdraw_fees(req.address.clone(), amount)
                .await?,
        ),
    };
    let absolute_fees = fees.amount();

    info!("Attempting withdraw with fees: {fees:?}");

    let operation_id = wallet_module
        .withdraw(req.address, amount, fees, ())
        .await?;

    let mut updates = wallet_module
        .subscribe_withdraw_updates(operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        info!("Update: {update:?}");

        match update {
            WithdrawState::Succeeded(txid) => {
                return Ok(Json(WithdrawResponse {
                    txid: txid.to_hex(),
                    fees_sat: absolute_fees.to_sat(),
                }));
            }
            WithdrawState::Failed(e) => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!("Withdraw failed: {:?}", e),
                ));
            }
            _ => continue,
        };
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Update stream ended without outcome"),
    ))
}

#[axum_macros::debug_handler]
pub async fn handle_discoverversion(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "version": state.fm.discover_common_api_version().await? })))
}

#[axum_macros::debug_handler]
pub async fn handle_backup(
    State(state): State<AppState>,
    Json(req): Json<BackupRequest>,
) -> Result<Json<()>, AppError> {
    state
        .fm
        .backup_to_federation(Metadata::from_json_serialized(req.metadata))
        .await?;
    Ok(Json(()))
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
pub async fn handle_listoperations(
    State(state): State<AppState>,
    Json(req): Json<ListOperationsRequest>,
) -> Result<Json<Value>, AppError> {
    const ISO8601_CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
        .set_formatted_components(iso8601::FormattedComponents::DateTime)
        .encode();
    let operations = state
        .fm
        .operation_log()
        .list_operations(req.limit, None)
        .await
        .into_iter()
        .map(|(k, v)| {
            let creation_time = OffsetDateTime::from_unix_timestamp(
                k.creation_time
                    .duration_since(UNIX_EPOCH)
                    .expect("Couldn't convert time from SystemTime to timestamp")
                    .as_secs() as i64,
            )
            .expect("Couldn't convert time from SystemTime to OffsetDateTime")
            .format(&iso8601::Iso8601::<ISO8601_CONFIG>)
            .expect("Couldn't format OffsetDateTime as ISO8601");

            OperationOutput {
                id: k.operation_id,
                creation_time,
                operation_kind: v.operation_module_kind().to_owned(),
                operation_meta: v.meta(),
                outcome: v.outcome(),
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "operations": operations,
    })))
}

#[axum_macros::debug_handler]
pub async fn handle_module() -> Result<(), AppError> {
    // TODO: Figure out how to impl this
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_config(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let config = state.fm.get_config_json();
    Ok(Json(serde_json::to_value(config).expect("Client config is serializable")))
}
