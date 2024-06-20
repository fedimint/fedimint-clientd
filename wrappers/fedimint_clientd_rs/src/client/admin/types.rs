#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::client::types::{FederationInfo, OperationOutput};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListOperationsRequest {
    pub limit: u64,
    pub federationId: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub inviteCode: String,
    pub useManualSecret: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscoverVersionRequest {
    pub threshold: usize,
}

pub type InfoResponse = HashMap<String, FederationInfo>;

pub type DiscoverVersionResponse = HashMap<String, Value>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOperationsResponse {
    pub operations: Vec<OperationOutput>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationIdsResponse {
    pub federation_ids: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinResponse {
    pub this_federation_id: String,
    pub federation_ids: Vec<String>,
}

pub struct JoinOptions {
    pub invite_code: String,
    pub use_default_gateway: bool,
    pub set_active_federation_id: bool,
    pub use_manual_secret: bool,
}

impl JoinOptions {
    pub fn new(invite_code: String) -> Self {
        JoinOptions {
            invite_code,
            set_active_federation_id: false,
            use_default_gateway: false,
            use_manual_secret: false,
        }
    }

    pub fn set_active_federation_id(mut self) -> Self {
        self.set_active_federation_id = true;
        self
    }

    pub fn use_default_gateway(mut self) -> Self {
        self.use_default_gateway = true;
        self
    }

    pub fn use_manual_secret(mut self) -> Self {
        self.use_manual_secret = true;
        self
    }
}
