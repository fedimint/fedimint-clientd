use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use nostr_sdk::secp256k1::SecretKey;
use nostr_sdk::Keys;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyManager {
    server_key: SecretKey,
    user_key: SecretKey,
    #[serde(default)]
    pub sent_info: bool,
}

impl KeyManager {
    pub fn new(keys_file: &str) -> Result<Self> {
        let path = Path::new(keys_file);
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).context("Failed to parse JSON")
            }
            Err(_) => {
                let keys = Self::generate()?;
                Self::write_keys(&keys, path)?;
                Ok(keys)
            }
        }
    }

    fn generate() -> Result<Self> {
        let server_keys = Keys::generate();
        let server_key = server_keys.secret_key()?;
        let user_keys = Keys::generate();
        let user_key = user_keys.secret_key()?;
        Ok(Self {
            server_key: **server_key,
            user_key: **user_key,
            sent_info: false,
        })
    }

    fn write_keys(keys: &Self, path: &Path) -> Result<()> {
        let json_str = serde_json::to_string(keys).context("Failed to serialize data")?;
        if let Some(parent) = path.parent() {
            create_dir_all(parent).context("Failed to create directory")?;
        }
        let mut file = File::create(path).context("Failed to create file")?;
        file.write_all(json_str.as_bytes())
            .context("Failed to write to file")?;
        Ok(())
    }

    pub fn server_keys(&self) -> Keys {
        Keys::new(self.server_key.into())
    }

    pub fn user_keys(&self) -> Keys {
        Keys::new(self.user_key.into())
    }
}
