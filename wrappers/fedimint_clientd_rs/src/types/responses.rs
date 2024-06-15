#![allow(non_snake_case)]

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::{FederationInfo, Gateway, LnReceiveState, NotesJson, OperationOutput};

pub type InfoResponse = HashMap<String, FederationInfo>;

pub type DiscoverVersionResponse = HashMap<String, Value>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationsResponse {
    pub operations: Vec<OperationOutput>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FederationIdsResponse {
    pub federation_ids: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JoinResponse {
    pub this_federation_id: String,
    pub federation_ids: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceResponse {
    pub operation_id: String,
    pub invoice: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LightningPaymentResponse {
    pub status: LnReceiveState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceResponse {
    pub status: LnReceiveState,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LightningPayResponse {
    pub operation_id: String,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: u64,
    pub preimage: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PayType {
    // Payment from this client to another user within the federation
    Internal(String),
    // Payment from this client to another user, facilitated by a gateway
    Lightning(String),
}

pub type ListGatewaysResponse = Vec<Gateway>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DecodeNotesResponse {
    pub notes_json: NotesJson,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EncodeNotesResponse {
    pub notes: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReissueResponse {
    pub amount_msat: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpendResponse {
    pub operation: String,
    pub notes: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub amount_msat: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SplitResponse {
    pub notes: HashMap<u64, String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombineResponse {
    pub notes: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateDepositAddressResponse {
    pub operationId: String,
    pub address: String,
}

#[derive(Deserialize, Debug)]
pub struct AwaitDepositResponseConfirmed {
    pub btc_transaction: BTCTransaction,
    pub out_idx: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BTCInput {
    pub previous_output: String,
    pub script_sig: String,
    pub sequence: u64,
    pub witness: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BTCOutput {
    pub value: u64,
    pub script_pubkey: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BTCTransaction {
    pub version: usize,
    pub lock_time: usize,
    pub input: Vec<BTCInput>,
    pub output: Vec<BTCOutput>,
}

#[derive(Deserialize, Debug)]
pub struct AwaitedDepositConfirmed {
    pub Confirmed: AwaitDepositResponseConfirmed,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AwaitedDepositFailed {
    pub Failed: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct BitcoinTransactionData {
    /// The bitcoin transaction is saved as soon as we see it so the transaction
    /// can be re-transmitted if it's evicted from the mempool.
    pub btc_transaction: Value,
    /// Index of the deposit output
    pub out_idx: u32,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub enum DepositState {
    WaitingForTransaction,
    WaitingForConfirmation(BitcoinTransactionData),
    Confirmed(BitcoinTransactionData),
    Claimed(BitcoinTransactionData),
    Failed(String),
}

#[derive(Deserialize, Clone, Debug)]
pub struct AwaitDepositResponse {
    pub status: DepositState,
}
