#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

use crate::client::types::LnReceiveState;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceResponse {
    pub operation_id: String,
    pub invoice: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightningPaymentResponse {
    pub status: LnReceiveState,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceResponse {
    pub status: LnReceiveState,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightningPayResponse {
    pub operation_id: String,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: u64,
    pub preimage: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayType {
    // Payment from this client to another user within the federation
    Internal(String),
    // Payment from this client to another user, facilitated by a gateway
    Lightning(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub gatewayId: String,
    pub federationId: String,
    pub amountMsat: u64,
    pub description: String,
    pub expiryTime: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateTweakedInvoiceRequest {
    pub gatewayId: String,
    pub federationId: String,
    pub amountMsat: u64,
    pub tweak: u64,
    pub description: String,
    pub expiryTime: Option<u64>,
    pub externalPubkey: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClaimPubkeyTweakRequest {
    pub federationId: String,
    pub privateKey: String,
    pub tweaks: Vec<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AwaitInvoiceRequest {
    pub federationId: String,
    pub operationId: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LightningPayRequest {
    pub federationId: String,
    pub gatewayId: String,
    pub paymentInfo: String,
    pub amountMsat: Option<u64>,
    pub LightningurlComment: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListGatewaysRequest {
    pub federationId: String,
}

pub struct InvoiceOptions {
    pub amount_msat: u64,
    pub description: String,
    pub expiry_time: Option<u64>,
}

impl InvoiceOptions {
    pub fn new() -> Self {
        InvoiceOptions {
            amount_msat: 0,
            description: "".to_string(),
            expiry_time: None,
        }
    }

    pub fn msats(mut self, msats: u64) -> Self {
        self.amount_msat = msats;
        self
    }

    pub fn sats(mut self, sats: u64) -> Self {
        self.amount_msat = sats * 1000;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn expiry_time(mut self, expiry_time: u64) -> Self {
        self.expiry_time = Some(expiry_time);
        self
    }
}

impl Default for InvoiceOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TweakedInvoiceOptions {
    pub amount_msat: u64,
    pub tweak: u64,
    pub description: String,
    pub external_pubkey: String,
    pub expiry_time: Option<u64>,
}

impl TweakedInvoiceOptions {
    pub fn new() -> Self {
        TweakedInvoiceOptions {
            amount_msat: 0,
            tweak: 0,
            external_pubkey: "".to_string(),
            description: "".to_string(),
            expiry_time: None,
        }
    }

    pub fn msats(mut self, msats: u64) -> Self {
        self.amount_msat = msats;
        self
    }

    pub fn sats(mut self, sats: u64) -> Self {
        self.amount_msat = sats * 1000;
        self
    }

    pub fn tweak(mut self, tweak: u64) -> Self {
        self.tweak = tweak;
        self
    }

    pub fn external_pubkey(mut self, external_pubkey: String) -> Self {
        self.external_pubkey = external_pubkey;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn expiry_time(mut self, expiry_time: u64) -> Self {
        self.expiry_time = Some(expiry_time);
        self
    }
}

impl Default for TweakedInvoiceOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PayOptions {
    pub payment_info: String,
    pub amount_msat: Option<u64>,
    pub lightningurl_comment: Option<String>,
}

impl PayOptions {
    pub fn new() -> Self {
        PayOptions {
            amount_msat: None,
            payment_info: "".to_string(),
            lightningurl_comment: None,
        }
    }

    pub fn msats(mut self, msats: u64) -> Self {
        self.amount_msat = Some(msats);
        self
    }

    pub fn sats(mut self, sats: u64) -> Self {
        self.amount_msat = Some(sats * 1000);
        self
    }

    pub fn payment_info(mut self, payment_info: String) -> Self {
        self.payment_info = payment_info;
        self
    }

    pub fn comment(mut self, comment: String) -> Self {
        self.lightningurl_comment = Some(comment);
        self
    }
}

impl Default for PayOptions {
    fn default() -> Self {
        Self::new()
    }
}
