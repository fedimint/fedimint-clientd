#![allow(non_snake_case)]

use serde::Serialize;

#[derive(Serialize)]
pub struct ListOperationsRequest {
    pub limit: u64,
    pub federationId: String,
}

#[derive(Serialize)]
pub struct JoinRequest {
    pub inviteCode: String,
    pub useManualSecret: bool,
}

#[derive(Serialize)]
pub struct DiscoverVersionRequest {
    pub threshold: usize,
}

#[derive(Serialize)]
pub struct CreateInvoiceRequest {
    pub gatewayId: String,
    pub federationId: String,
    pub amountMsat: u64,
    pub description: String,
    pub expiryTime: Option<u64>,
}

pub struct CreateInvoiceArgs {
    pub amount_msat: u64,
    pub description: String,
    pub expiry_time: Option<u64>,
}

#[derive(Serialize)]
pub struct CreateTweakedInvoiceRequest {
    pub gatewayId: String,
    pub federationId: String,
    pub amountMsat: u64,
    pub tweak: u64,
    pub description: String,
    pub expiryTime: Option<u64>,
    pub externalPubkey: String,
}

#[derive(Serialize)]
pub struct ClaimPubkeyTweakRequest {
    pub federationId: String,
    pub privateKey: String,
    pub tweaks: Vec<usize>,
}

pub struct ClaimPubkeyTweakArgs {
    pub private_key: String,
    pub tweaks: Vec<usize>,
}

#[derive(Serialize)]
pub struct AwaitInvoiceRequest {
    pub federationId: String,
    pub operationId: String,
}

#[derive(Serialize)]
pub struct LightningPayRequest {
    pub federationId: String,
    pub gatewayId: String,
    pub paymentInfo: String,
    pub amountMsat: Option<u64>,
    pub LightningurlComment: Option<String>,
}

#[derive(Serialize)]
pub struct ListGatewaysRequest {
    pub federationId: String,
}

#[derive(Serialize)]
pub struct DecodeNotesRequest {
    pub notes: String,
}

#[derive(Serialize)]
pub struct EncodeNotesRequest {
    pub notesJsonStr: String,
}

#[derive(Serialize)]
pub struct ReissueRequest {
    pub federationId: String,
    pub notes: String,
}

#[derive(Serialize)]
pub struct SpendRequest {
    pub federationId: String,
    pub amountMsat: u64,
    pub allowOverpay: bool,
    pub timeout: u64,
    pub includeInvite: bool,
}

#[derive(Serialize)]
pub struct SpendArgs {
    pub amountMsat: u64,
    pub allowOverpay: bool,
    pub timeout: u64,
    pub includeInvite: bool,
}

#[derive(Serialize)]
pub struct ValidateRequest {
    pub federationId: String,
    pub notes: String,
}

#[derive(Serialize)]
pub struct SplitRequest {
    pub notes: String,
}

#[derive(Serialize)]
pub struct CombineRequest {
    pub notesVec: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateDepositAddressRequest {
    pub federationId: String,
    pub timeout: u64,
}

#[derive(Serialize)]
pub struct AwaitDepositRequest {
    pub federationId: String,
    pub operationId: String,
}
