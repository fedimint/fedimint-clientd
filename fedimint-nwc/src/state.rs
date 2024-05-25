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
use nostr_sdk::{Client, Event, EventBuilder, EventId, Filter, JsonUtil, Keys, Kind, Timestamp};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::Cli;
use crate::nwc::{handle_nwc_request, METHODS};

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: MultiMint,
    pub user_keys: Keys,
    pub server_keys: Keys,
    pub sent_info: bool,
    pub nostr_client: Client,
    pub active_requests: Arc<Mutex<BTreeSet<EventId>>>,
}

impl AppState {
    pub async fn new(fm_db_path: PathBuf, keys_file: &str, relays: &str) -> Result<Self> {
        let clients = MultiMint::new(fm_db_path).await?;
        clients.update_gateway_caches().await?;

        info!("Setting up nostr client...");
        let keys = Nip47Keys::load_or_generate(keys_file)?;
        let lines = relays.split(',').collect::<Vec<_>>();
        let relays = lines
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        let nostr_client = Client::new(&keys.server_keys());
        info!("Adding relays...");
        for relay in relays {
            nostr_client.add_relay(relay).await?;
        }
        info!("Setting NWC subscription...");
        let subscription = setup_subscription(&keys);
        nostr_client.subscribe(vec![subscription], None).await;

        let active_requests = Arc::new(Mutex::new(BTreeSet::new()));

        Ok(Self {
            multimint: clients,
            user_keys: keys.user_keys(),
            server_keys: keys.server_keys(),
            sent_info: keys.sent_info,
            nostr_client,
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

    pub async fn broadcast_info_event(&mut self) -> Result<()> {
        let content = METHODS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let info =
            EventBuilder::new(Kind::WalletConnectInfo, content, []).to_event(&self.server_keys)?;
        let res = self
            .nostr_client
            .send_event(info)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e));

        if res.is_ok() {
            self.sent_info = true;
            info!("Sent info event...");
        } else {
            error!("Failed to send info event: {}", res.err().unwrap());
        }

        Ok(())
    }

    pub async fn handle_event(&self, event: Event) {
        if event.kind == Kind::WalletConnectRequest && event.verify().is_ok() {
            info!("Received event: {}", event.as_json());
            let event_id = event.id;
            self.active_requests.lock().await.insert(event_id);

            match tokio::time::timeout(Duration::from_secs(60), handle_nwc_request(&self, event))
                .await
            {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => error!("Error processing request: {e}"),
                Err(e) => error!("Timeout error: {e}"),
            }

            self.active_requests.lock().await.remove(&event_id);
        } else {
            error!("Invalid event: {}", event.as_json());
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

fn setup_subscription(keys: &Nip47Keys) -> Filter {
    Filter::new()
        .kinds(vec![Kind::WalletConnectRequest])
        .author(keys.user_keys().public_key())
        .pubkey(keys.server_keys().public_key())
        .since(Timestamp::now())
}
