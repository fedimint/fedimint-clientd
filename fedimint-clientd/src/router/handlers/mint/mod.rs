use multimint::fedimint_core::TieredMulti;
use multimint::fedimint_mint_client::SpendableNote;
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
    federation_id_prefix: String,
    notes: TieredMulti<SpendableNote>,
}
