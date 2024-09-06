use fedimint_api_client::api::net::Connector;
use fedimint_core::config::FederationId;
use fedimint_core::encoding::{Decodable, Encodable};
use fedimint_core::invite_code::InviteCode;
use fedimint_core::{impl_db_lookup, impl_db_record};
use fedimint_ln_common::lightning_invoice::RoutingFees;
use fedimint_ln_common::serde_routing_fees;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum DbKeyPrefix {
    FederationConfig = 0x04,
}

impl std::fmt::Display for DbKeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Encodable, Decodable, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FederationIdKey {
    pub id: FederationId,
}

#[derive(Debug, Encodable, Decodable)]
pub struct FederationIdKeyPrefix;

#[derive(Debug, Clone, Eq, PartialEq, Encodable, Decodable, Serialize, Deserialize)]
pub struct FederationConfig {
    pub invite_code: InviteCode,
    // Unique integer identifier per-federation that is assigned when the gateways joins a
    // federation.
    #[serde(alias = "mint_channel_id")]
    pub federation_index: u64,
    pub timelock_delta: u64,
    #[serde(with = "serde_routing_fees")]
    pub fees: RoutingFees,
    pub connector: Connector,
}

impl_db_record!(
    key = FederationIdKey,
    value = FederationConfig,
    db_prefix = DbKeyPrefix::FederationConfig,
);

impl_db_lookup!(key = FederationIdKey, query_prefix = FederationIdKeyPrefix);
