#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateDepositAddressRequest {
    pub federationId: String,
    pub timeout: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitDepositRequest {
    pub federationId: String,
    pub operationId: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDepositAddressResponse {
    pub operationId: String,
    pub address: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitDepositResponseConfirmed {
    pub btc_transaction: BTCTransaction,
    pub out_idx: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BTCInput {
    pub previous_output: String,
    pub script_sig: String,
    pub sequence: u64,
    pub witness: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BTCOutput {
    pub value: u64,
    pub script_pubkey: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BTCTransaction {
    pub version: usize,
    pub lock_time: usize,
    pub input: Vec<BTCInput>,
    pub output: Vec<BTCOutput>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitedDepositConfirmed {
    pub Confirmed: AwaitDepositResponseConfirmed,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitedDepositFailed {
    pub Failed: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitcoinTransactionData {
    /// The bitcoin transaction is saved as soon as we see it so the transaction
    /// can be re-transmitted if it's evicted from the mempool.
    pub btc_transaction: Value,
    /// Index of the deposit output
    pub out_idx: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DepositState {
    WaitingForTransaction,
    WaitingForConfirmation(BitcoinTransactionData),
    Confirmed(BitcoinTransactionData),
    Claimed(BitcoinTransactionData),
    Failed(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitDepositResponse {
    pub status: DepositState,
}
