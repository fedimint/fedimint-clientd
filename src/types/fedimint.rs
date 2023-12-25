use fedimint_core::{config::FederationId, core::OperationId, Amount, TieredSummary};
use fedimint_mint_client::OOBNotes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub federation_id: FederationId,
    pub network: String,
    pub meta: BTreeMap<String, String>,
    pub total_amount_msat: Amount,
    pub total_num_notes: usize,
    pub denominations_msat: TieredSummary,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReissueRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReissueResponse {
    pub amount: Amount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpendRequest {
    pub amount: Amount,
    pub allow_overpay: bool,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpendResponse {
    pub operation: OperationId,
    pub notes: OOBNotes,
}
