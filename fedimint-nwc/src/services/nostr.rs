use anyhow::Result;
use nostr_sdk::{Client, EventBuilder, EventId, Kind, RelayPoolNotification};
use tokio::sync::broadcast::Receiver;

use crate::managers::key::KeyManager;
use crate::nwc::METHODS;

#[derive(Debug, Clone)]
pub struct NostrService {
    client: Client,
}

impl NostrService {
    pub async fn new(key_manager: &KeyManager, relays: &str) -> Result<Self> {
        let client = Client::new(&key_manager.server_keys());
        Self::add_relays(&client, relays).await?;
        Ok(Self { client })
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

    pub async fn broadcast_info_event(&self, keys: &KeyManager) -> Result<EventId> {
        let content = METHODS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let info = EventBuilder::new(Kind::WalletConnectInfo, content, [])
            .to_event(&keys.server_keys())?;
        self.client
            .send_event(info)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))
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
}
