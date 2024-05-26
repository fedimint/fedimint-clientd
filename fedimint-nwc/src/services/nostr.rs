use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use nostr_sdk::secp256k1::SecretKey;
use nostr_sdk::{Client, Event, EventBuilder, JsonUtil, Keys, Kind, RelayPoolNotification};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Receiver;
use tracing::info;

use crate::nwc::METHODS;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NostrService {
    #[serde(skip)]
    client: Client,
    server_key: SecretKey,
    user_key: SecretKey,
    #[serde(default)]
    pub sent_info: bool,
}

impl NostrService {
    pub async fn new(keys_file: &str, relays: &str) -> Result<Self> {
        let path = Path::new(keys_file);
        let (server_key, user_key) = match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let keys: Self = serde_json::from_reader(reader).context("Failed to parse JSON")?;
                (keys.server_key, keys.user_key)
            }
            Err(_) => {
                let (server_key, user_key) = Self::generate_keys()?;
                Self::write_keys(server_key, user_key, path)?;
                (server_key, user_key)
            }
        };

        let client = Client::new(&Keys::new(server_key.into()));
        Self::add_relays(&client, relays).await?;
        Ok(Self {
            client,
            server_key,
            user_key,
            sent_info: false,
        })
    }

    fn generate_keys() -> Result<(SecretKey, SecretKey)> {
        let server_keys = Keys::generate();
        let server_key = server_keys.secret_key()?;
        let user_keys = Keys::generate();
        let user_key = user_keys.secret_key()?;
        Ok((**server_key, **user_key))
    }

    fn write_keys(server_key: SecretKey, user_key: SecretKey, path: &Path) -> Result<()> {
        let keys = Self {
            server_key,
            user_key,
            sent_info: false,
            client: Client::new(&Keys::new(server_key.into())), /* Dummy client for struct
                                                                 * initialization */
        };
        let json_str = serde_json::to_string(&keys).context("Failed to serialize data")?;
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

    async fn add_relays(client: &Client, relays: &str) -> Result<()> {
        let lines = relays.split(',').collect::<Vec<_>>();
        let relays = lines
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        for relay in relays {
            client.add_relay(relay).await?;
        }
        Ok(())
    }

    pub async fn broadcast_info_event(&self) -> Result<(), anyhow::Error> {
        let content = METHODS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let info = EventBuilder::new(Kind::WalletConnectInfo, content, [])
            .to_event(&self.server_keys())?;
        info!("Broadcasting info event: {}", info.as_json());
        let event_id = self.client.send_event(info).await?;
        info!("Broadcasted info event: {}", event_id);
        Ok(())
    }

    pub async fn connect(&self) -> () {
        self.client.connect().await
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.client
            .disconnect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to disconnect: {}", e))
    }

    pub fn notifications(&self) -> Receiver<RelayPoolNotification> {
        self.client.notifications()
    }

    pub fn is_nwc_event(&self, event: &Event) -> bool {
        event.kind == Kind::WalletConnectRequest
            && event.verify().is_ok()
            && event.pubkey == self.user_keys().public_key()
    }
}
