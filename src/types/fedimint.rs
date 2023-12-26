use bitcoin::Address;
use fedimint_core::{config::FederationId, core::OperationId, Amount, TieredSummary};
use fedimint_ln_client::PayType;
use fedimint_mint_client::OOBNotes;
use fedimint_wallet_client::DepositState;
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

#[derive(Debug, Deserialize)]
pub struct AwaitInvoiceRequest {
    pub operation_id: OperationId,
}

#[derive(Debug, Deserialize)]
pub struct LnPayRequest {
    pub payment_info: String,
    pub amount_msat: Option<Amount>,
    pub finish_in_background: bool,
    pub lnurl_comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LnPayResponse {
    pub operation_id: OperationId,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: Amount,
}

#[derive(Debug, Deserialize)]
pub struct AwaitLnPayRequest {
    pub operation_id: OperationId,
}

#[derive(Debug, Serialize)]
pub struct AwaitLnPayResponse {
    pub operation_id: OperationId,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: Amount,
}

#[derive(Debug, Deserialize)]
pub struct SwitchGatewayRequest {
    pub gateway_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositAddressRequest {
    pub timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct DepositAddressResponse {
    pub operation_id: OperationId,
    pub address: Address,
}

#[derive(Debug, Deserialize)]
pub struct AwaitDepositRequest {
    pub operation_id: OperationId,
}

#[derive(Debug, Serialize)]
pub struct AwaitDepositResponse {
    pub status: DepositState,
}
