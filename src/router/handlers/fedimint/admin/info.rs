use std::collections::{BTreeMap, HashMap};

use anyhow::Error;
use axum::{extract::State, Json};
use fedimint_core::{config::FederationId, Amount, TieredSummary};
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;
use multimint::MultiMint;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub network: String,
    pub meta: BTreeMap<String, String>,
    pub total_amount_msat: Amount,
    pub total_num_notes: usize,
    pub denominations_msat: TieredSummary,
}

async fn _info(multimint: MultiMint) -> Result<HashMap<FederationId, InfoResponse>, Error> {
    let mut info = HashMap::new();

    for (id, client) in multimint.clients.lock().await.iter() {
        let mint_client = client.get_first_module::<MintClientModule>();
        let wallet_client = client.get_first_module::<WalletClientModule>();
        let summary = mint_client
            .get_wallet_summary(
                &mut client
                    .db()
                    .begin_transaction_nc()
                    .await
                    .to_ref_with_prefix_module_id(1),
            )
            .await;

        info.insert(
            *id,
            InfoResponse {
                network: wallet_client.get_network().to_string(),
                meta: client.get_config().global.meta.clone(),
                total_amount_msat: summary.total_amount(),
                total_num_notes: summary.count_items(),
                denominations_msat: summary,
            },
        );
    }

    Ok(info)
}

pub async fn handle_ws(state: AppState, _v: Value) -> Result<Value, AppError> {
    let info = _info(state.multimint).await?;
    let info_json = json!(info);
    Ok(info_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<HashMap<FederationId, InfoResponse>>, AppError> {
    let info = _info(state.multimint).await?;
    Ok(Json(info))
}
