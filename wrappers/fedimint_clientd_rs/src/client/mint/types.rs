#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::types::NotesJson;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodeNotesRequest {
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EncodeNotesRequest {
    pub notesJsonStr: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReissueRequest {
    pub federationId: String,
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpendRequest {
    pub federationId: String,
    pub amountMsat: u64,
    pub allowOverpay: bool,
    pub timeout: u64,
    pub includeInvite: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpendArgs {
    pub amountMsat: u64,
    pub allowOverpay: bool,
    pub timeout: u64,
    pub includeInvite: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub federationId: String,
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SplitRequest {
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CombineRequest {
    pub notesVec: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeNotesResponse {
    pub notes_json: NotesJson,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeNotesResponse {
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReissueResponse {
    pub amount_msat: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendResponse {
    pub operation: String,
    pub notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub amount_msat: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitResponse {
    pub notes: HashMap<u64, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineResponse {
    pub notes: String,
}

pub struct SpendOptions {
    pub amount_msat: u64,
    pub allow_overpay: bool,
    pub timeout: u64,
    pub include_invite: bool,
}

impl SpendOptions {
    pub fn new() -> Self {
        SpendOptions {
            amount_msat: 0,
            allow_overpay: false,
            timeout: 0,
            include_invite: false,
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

    pub fn allow_overpay(mut self, allow_overpay: bool) -> Self {
        self.allow_overpay = allow_overpay;
        self
    }

    pub fn include_invite(mut self, include_invite: bool) -> Self {
        self.include_invite = include_invite;
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for SpendOptions {
    fn default() -> Self {
        Self::new()
    }
}
