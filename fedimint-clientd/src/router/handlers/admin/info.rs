use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Error};
use axum::extract::State;
use axum::Json;
use multimint::cdk::nuts::{CurrencyUnit, Proof};
use multimint::cdk::UncheckedUrl;
use multimint::fedimint_core::config::FederationId;
use multimint::fedimint_core::{Amount, TieredSummary};
use multimint::fedimint_mint_client::MintClientModule;
use multimint::fedimint_wallet_client::WalletClientModule;
use multimint::MultiMint;
use serde::Serialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoResponse {
    pub fedimint_clients: HashMap<FederationId, FedimintClientInfo>,
    pub cashu_wallets: HashMap<UncheckedUrl, CashuWalletInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashuWalletInfo {
    pub total_balance: Amount,
    pub proofs: Vec<Proof>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FedimintClientInfo {
    pub network: String,
    pub meta: BTreeMap<String, String>,
    pub total_amount_msat: Amount,
    pub total_num_notes: usize,
    pub denominations_msat: TieredSummary,
}

async fn _info(multimint: MultiMint) -> Result<InfoResponse, Error> {
    let mut fedimint_clients_info = HashMap::new();

    for (id, client) in multimint.fedimint_clients.lock().await.iter() {
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

        fedimint_clients_info.insert(
            *id,
            FedimintClientInfo {
                network: wallet_client.get_network().to_string(),
                meta: client.get_config().global.meta.clone(),
                total_amount_msat: summary.total_amount(),
                total_num_notes: summary.count_items(),
                denominations_msat: summary,
            },
        );
    }

    let cashu_wallet = multimint.cashu_wallet.lock().await;
    let cashu_mints = cashu_wallet.mint_balances().await?;
    let mut cashu_wallets_info = HashMap::new();
    for (mint_url, balance) in cashu_mints.iter() {
        let proofs = cashu_wallet.get_proofs(mint_url.to_owned()).await?;
        let total_balance = balance
            .get(&CurrencyUnit::Sat)
            .ok_or(anyhow!("Sat not found"))?;
        cashu_wallets_info.insert(
            mint_url.clone(),
            CashuWalletInfo {
                total_balance: Amount::from_sats((*total_balance).into()),
                proofs: proofs.unwrap_or_default(),
            },
        );
    }

    Ok(InfoResponse {
        fedimint_clients: fedimint_clients_info,
        cashu_wallets: cashu_wallets_info,
    })
}

pub async fn handle_ws(state: AppState, _v: Value) -> Result<Value, AppError> {
    let info = _info(state.multimint).await?;
    let info_json = json!(info);
    Ok(info_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(State(state): State<AppState>) -> Result<Json<InfoResponse>, AppError> {
    let info = _info(state.multimint).await?;
    Ok(Json(info))
}
