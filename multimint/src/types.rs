use std::collections::BTreeMap;

use fedimint_core::config::FederationId;
use fedimint_core::{Amount, TieredSummary};
use serde::Serialize;

/// InfoResponse for getting the Federation Config info
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub federation_id: FederationId,
    pub network: String,
    pub meta: BTreeMap<String, String>,
    pub total_amount_msat: Amount,
    pub total_num_notes: usize,
    pub denominations_msat: TieredSummary,
}
