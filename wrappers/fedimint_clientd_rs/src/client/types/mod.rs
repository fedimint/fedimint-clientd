#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FederationInfo {
    pub network: String,
    pub meta: HashMap<String, String>,
    pub total_amount_msat: u64,
    pub total_num_notes: usize,
    pub denominations_msat: HashMap<String, u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationOutput {
    pub id: Value,
    pub creation_time: Value,
    pub operation_kind: Value,
    pub operation_meta: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LnReceiveState {
    Created,
    WaitingForPayment { invoice: String, timeout: usize },
    Canceled { reason: String },
    Funded,
    AwaitingFunds,
    Claimed,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gateway {
    pub federation_id: String,
    pub info: GatewayInfo,
    pub vetted: bool,
    pub ttl: GatewayTTL,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub api: String,
    pub fees: GatewayFees,
    pub gateway_id: String,
    pub gateway_redeem_key: String,
    pub lightning_alias: String,
    pub mint_channel_id: u64,
    pub node_pub_key: String,
    pub route_hints: Vec<Value>,
    pub supports_private_payments: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayTTL {
    pub nanos: u32,
    pub secs: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayFees {
    pub base_msat: u32,
    pub proportional_millionths: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NotesJson {
    pub federation_id_prefix: String,
    pub notes: HashMap<String, Vec<Note>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub signature: String,
    pub spend_key: String,
}
