use std::str::FromStr;

use fedimint_core::TieredMulti;
use fedimint_mint_client::SpendableNote;
use serde::{Deserialize, Serialize};

pub mod combine;
pub mod decode_notes;
pub mod encode_notes;
pub mod reissue;
pub mod spend;
pub mod split;
pub mod validate;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OOBNotesJson {
    pub federation_id_prefix: String,
    pub notes: TieredMulti<SpendableNote>,
}

impl FromStr for OOBNotesJson {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|e| anyhow::anyhow!("{}", e))
    }
}
