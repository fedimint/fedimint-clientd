use std::collections::BTreeSet;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use multimint::fedimint_core::api::InviteCode;
use multimint::MultiMint;
use nostr_sdk::secp256k1::SecretKey;
use nostr_sdk::{EventId, Keys};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::Cli;

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: MultiMint,
    pub user_keys: Keys,
    pub server_keys: Keys,
    pub active_requests: Arc<Mutex<BTreeSet<EventId>>>,
}

impl AppState {
    pub async fn new(fm_db_path: PathBuf, keys_file: &str) -> Result<Self> {
        let clients = MultiMint::new(fm_db_path).await?;
        clients.update_gateway_caches().await?;

        let keys = Nip47Keys::load_or_generate(keys_file)?;

        let active_requests = Arc::new(Mutex::new(BTreeSet::new()));

        Ok(Self {
            multimint: clients,
            user_keys: keys.user_keys(),
            server_keys: keys.server_keys(),
            active_requests,
        })
    }

    pub async fn load_manual_secret(cli: &Cli) -> Option<String> {
        cli.manual_secret
            .clone()
            .or_else(|| std::env::var("FEDIMINT_CLIENTD_MANUAL_SECRET").ok())
    }

    pub async fn init_multimint(
        &mut self,
        invite_code: &str,
        manual_secret: Option<String>,
    ) -> Result<()> {
        match InviteCode::from_str(invite_code) {
            Ok(invite_code) => {
                let federation_id = self
                    .multimint
                    .register_new(invite_code, manual_secret)
                    .await?;
                info!("Created client for federation id: {:?}", federation_id);
                Ok(())
            }
            Err(e) => {
                error!("Invalid federation invite code: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn wait_for_active_requests(&self) {
        let requests = self.active_requests.lock().await;
        loop {
            if requests.is_empty() {
                break;
            }
            debug!("Waiting for {} requests to complete...", requests.len());
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Nip47Keys {
    server_key: SecretKey,
    user_key: SecretKey,
    #[serde(default)]
    sent_info: bool,
}

impl Nip47Keys {
    fn generate() -> Result<Self> {
        let server_keys = Keys::generate();
        let server_key = server_keys.secret_key()?;

        let user_keys = Keys::generate();
        let user_key = user_keys.secret_key()?;

        Ok(Nip47Keys {
            server_key: **server_key,
            user_key: **user_key,
            sent_info: false,
        })
    }

    fn load_or_generate(keys_file: &str) -> Result<Self> {
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

    fn write_keys(keys: &Nip47Keys, path: &Path) -> Result<()> {
        let json_str = serde_json::to_string(keys).context("Failed to serialize data")?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).context("Failed to create directory")?;
        }

        let mut file = File::create(path).context("Failed to create file")?;
        file.write_all(json_str.as_bytes())
            .context("Failed to write to file")?;
        Ok(())
    }

    fn server_keys(&self) -> Keys {
        Keys::new(self.server_key.into())
    }

    fn user_keys(&self) -> Keys {
        Keys::new(self.user_key.into())
    }
}
