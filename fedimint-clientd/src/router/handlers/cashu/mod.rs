use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

use crate::fedimint::mint::OOBNotesJson;

pub mod checkstate;
pub mod info;
pub mod keys;
pub mod keysets;
pub mod melt;
pub mod mint;
pub mod swap;

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
pub struct BlindedMessage {
    pub amount: u64,
    pub id: String,
    #[serde(rename = "B_")]
    pub b_: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlindSignature {
    pub amount: u64,
    pub id: String,
    #[serde(rename = "C_")]
    pub c_: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Proof {
    pub amount: u64,
    pub id: String,
    pub secret: String,
    #[serde(rename = "C")]
    pub c: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Token {
    pub mint: String,
    pub proofs: Vec<Proof>,
}

// Custom serialize function for CashuToken
fn serialize_cashu_token<S>(token: &CashuToken, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert the CashuToken struct to JSON string
    let token_json = serde_json::to_string(&token).map_err(serde::ser::Error::custom)?;
    // Encode the JSON string to base64 URL-safe format
    let base64_token_json = URL_SAFE.encode(token_json.trim());
    // Construct the serialized token format with version prefix
    let serialized_token = format!("cashuA{}", base64_token_json);

    // Serialize the string
    serializer.serialize_str(&serialized_token)
}

// Custom deserialize function for CashuToken
fn deserialize_cashu_token<'de, D>(deserializer: D) -> Result<CashuToken, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the input to a string
    let s = String::deserialize(deserializer)?;
    // Extract the base64 URL-safe encoded JSON part
    let base64_encoded = s.trim_start_matches("cashuA");
    // Decode the base64 URL-safe format to JSON string
    let decoded_json = URL_SAFE
        .decode(base64_encoded)
        .map_err(serde::de::Error::custom)?;
    let decoded_str = String::from_utf8(decoded_json).map_err(serde::de::Error::custom)?;

    // Deserialize the JSON string to CashuToken struct
    serde_json::from_str(&decoded_str).map_err(serde::de::Error::custom)
}

#[derive(Debug)]
pub struct CashuToken {
    pub token: Vec<Token>,
    pub unit: Option<Unit>,
    pub memo: Option<String>,
}

impl Serialize for CashuToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_cashu_token(self, serializer)
    }
}

impl<'de> Deserialize<'de> for CashuToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_cashu_token(deserializer)
    }
}

impl From<&OOBNotesJson> for CashuToken {
    fn from(notes: &OOBNotesJson) -> Self {
        let proofs = notes
            .notes
            .iter()
            .map(|(amount, note)| {
                note.iter()
                    .map(|note| Proof {
                        amount: amount.try_into_sats().unwrap(),
                        id: note.signature.0.to_string(),
                        secret: note.spend_key.display_secret().to_string(),
                        c: note.signature.0.to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        CashuToken {
            token: vec![Token {
                mint: notes.federation_id_prefix.to_string(),
                proofs,
            }],
            unit: None,
            memo: None,
        }
    }
}
