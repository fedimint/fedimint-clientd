use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use anyhow::anyhow;
use base64::Engine;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::KeyPair;
use multimint::fedimint_core::api::InviteCode;
use multimint::fedimint_core::config::{FederationId, FederationIdPrefix};
use multimint::fedimint_core::db::DatabaseValue;
use multimint::fedimint_core::module::registry::ModuleDecoderRegistry;
use multimint::fedimint_core::{Amount, TieredMulti};
use multimint::fedimint_mint_client::{OOBNotes, SpendableNote};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use tbs::Signature;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Proof {
    // Amount unassociated with the unit
    amount: u64,
    // keyset id -> FederationId
    id: String,
    // secret -> hex encoded spend key's secret key
    secret: String,
    // signature -> hex encoded BLS signature
    C: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Token {
    mint: String,
    proofs: Vec<Proof>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenV3 {
    pub token: Vec<Token>,
    pub unit: Option<String>,
    pub memo: Option<String>,
}

impl TokenV3 {
    /// Serializes the `Token` struct to a base64 URL-safe string without
    /// padding and with the version prefix.
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)
            .map_err(|e| serde_json::Error::custom(format!("Failed to serialize token: {}", e)))?;
        let base64_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());
        Ok(format!("cashuA{}", base64_token))
    }

    /// Deserializes a base64 URL-safe string without padding (with version
    /// prefix) back to a `Token` struct.
    pub fn deserialize(encoded: &str) -> Result<Self, serde_json::Error> {
        if !encoded.starts_with("cashuA") {
            return Err(serde_json::Error::custom("Invalid token format"));
        }
        let base64_token = &encoded[6..];
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(base64_token.as_bytes())
            .map_err(|e| {
                serde_json::Error::custom(format!("Failed to decode base64 token: {}", e))
            })?;
        let json = String::from_utf8(bytes).map_err(|e| {
            serde_json::Error::custom(format!("Failed to decode base64 token: {}", e))
        })?;
        serde_json::from_str(&json)
    }

    pub fn _from_oobnotes(notes: OOBNotes, invite_code: InviteCode) -> Result<Self, anyhow::Error> {
        let mut token = TokenV3 {
            token: vec![],
            // Always msats
            unit: Some("msat".to_string()),
            // Federation Invite Code
            memo: Some(invite_code.to_string()),
        };
        for (amount, note) in notes.notes().iter() {
            let mut proofs = vec![];
            for spendable_note in note.iter() {
                let proof = Proof {
                    amount: amount.msats,
                    // stick the federation id prefix here instead of keyset
                    id: notes.federation_id_prefix().to_string(),
                    secret: hex::encode(spendable_note.spend_key.secret_key().to_bytes()),
                    C: hex::encode(spendable_note.signature.to_bytes()),
                };
                proofs.push(proof);
            }
            token.token.push(Token {
                mint: notes.federation_id_prefix().to_string(),
                proofs,
            });
        }
        Ok(token)
    }

    fn _to_oobnotes(&self, modules: &ModuleDecoderRegistry) -> Result<OOBNotes, anyhow::Error> {
        let federation_id_prefix = match self.token.first().map(|t| &t.proofs[0].id) {
            Some(id) => FederationIdPrefix::from_str(id)?,
            None => return Err(anyhow!("No token found")),
        };
        let secp = Secp256k1::new();
        let mut notes_map = BTreeMap::<Amount, Vec<SpendableNote>>::new();
        for t in self.token.iter() {
            for proof in t.proofs.iter() {
                let signature_bytes = hex::decode(&proof.C)
                    .map_err(|e| anyhow!("Failed to decode spendable note signature: {}", e))?;
                let signature = Signature::from_bytes(&signature_bytes, modules)?;
                let secret_key_bytes = hex::decode(&proof.secret)
                    .map_err(|e| anyhow!("Failed to decode spendable note spend key: {}", e))?;
                let sk = SecretKey::from_bytes(&secret_key_bytes, modules)
                    .map_err(|e| anyhow!("Failed to decode spendable note spend key: {}", e))?;
                let spend_key = KeyPair::from_secret_key(&secp, &sk);
                let spendable_note = SpendableNote {
                    signature,
                    spend_key,
                };
                let amount = Amount::from_msats(proof.amount);
                notes_map.entry(amount).or_default().push(spendable_note);
            }
        }
        let tiered_notes = TieredMulti::new(notes_map);
        Ok(OOBNotes::new(federation_id_prefix, tiered_notes))
    }
}

impl FromStr for TokenV3 {
    type Err = serde_json::Error;

    /// Parses a string to create a `Token` struct.
    /// Assumes the string is a base64 URL-safe encoded JSON of the `Token` with
    /// `cashuA` prefix.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TokenV3::deserialize(s)
    }
}

impl fmt::Display for TokenV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.serialize() {
            Ok(serialized) => write!(f, "{}", serialized),
            Err(_) => Err(fmt::Error),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Msat,
    Sat,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Bolt11,
    Onchain,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Keyset {
    id: String,
    unit: Unit,
    active: bool,
}

impl From<FederationId> for Keyset {
    fn from(federation_id: FederationId) -> Self {
        let as_str = format!("00{}", federation_id.to_string());
        Keyset {
            id: as_str,
            unit: Unit::Msat,
            active: true,
        }
    }
}
