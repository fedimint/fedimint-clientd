use std::collections::BTreeMap;

use anyhow::Error;
use axum::{
    extract::{ws::Message, State},
    Json,
};
use fedimint_core::{config::FederationId, Amount, TieredSummary};
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub federation_id: FederationId,
    pub network: String,
    pub meta: BTreeMap<String, String>,
    pub total_amount_msat: Amount,
    pub total_num_notes: usize,
    pub denominations_msat: TieredSummary,
}

async fn _info(state: AppState) -> Result<InfoResponse, Error> {
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
    Ok(InfoResponse {
        federation_id: state.fm.federation_id(),
        network: wallet_client.get_network().to_string(),
        meta: state.fm.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    })
}

pub async fn handle_ws(_v: Value, state: AppState) -> Result<Message, AppError> {
    let info = _info(state).await?;
    let info_json = json!(info);
    Ok(Message::Text(info_json.to_string()))
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<InfoResponse>, AppError> {
    let info = _info(state).await?;
    Ok(Json(info))
}
