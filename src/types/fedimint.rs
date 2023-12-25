use fedimint_core::{config::FederationId, core::OperationId, Amount, TieredSummary};
use fedimint_mint_client::OOBNotes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Debug, Deserialize)]
pub struct ReissueRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ReissueResponse {
    pub amount_msat: Amount,
}

#[derive(Debug, Deserialize)]
pub struct SpendRequest {
    pub amount_msat: Amount,
    pub allow_overpay: bool,
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct SpendResponse {
    pub operation: OperationId,
    pub notes: OOBNotes,
}

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub amount_msat: Amount,
}

#[derive(Debug, Deserialize)]
pub struct SplitRequest {
    pub notes: OOBNotes,
}

#[derive(Debug, Serialize)]
pub struct SplitResponse {
    pub notes: BTreeMap<Amount, OOBNotes>,
}

#[derive(Debug, Deserialize)]
pub struct CombineRequest {
    pub notes: Vec<OOBNotes>,
}

#[derive(Debug, Serialize)]
pub struct CombineResponse {
    pub notes: OOBNotes,
}

#[derive(Debug, Deserialize)]
pub struct LnInvoiceRequest {
    pub amount_msat: Amount,
    pub description: String,
    pub expiry_time: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct LnInvoiceResponse {
    pub operation_id: OperationId,
    pub invoice: String,
}
