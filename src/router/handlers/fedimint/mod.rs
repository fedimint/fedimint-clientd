pub mod ln;
pub mod mint;
pub mod onchain;

use std::time::UNIX_EPOCH;

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{BackupRequest, InfoResponse, ListOperationsRequest, OperationOutput},
};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_client::backup::Metadata;
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;
use serde_json::{json, Value};
use time::{format_description::well_known::iso8601, OffsetDateTime};

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
    // TODO: unimplemented in cli
    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Not implemented"),
    ))
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
