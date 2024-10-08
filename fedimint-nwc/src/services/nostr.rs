use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use nostr::nips::nip04;
use nostr::nips::nip47::{NostrWalletConnectURI, Response};
use nostr_sdk::secp256k1::SecretKey;
use nostr_sdk::{
    Client, Event, EventBuilder, EventId, Filter, JsonUtil, Keys, Kind, RelayPoolNotification, Tag,
    Timestamp, Url,
};
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
    pub keys_file_path: PathBuf,
    pub relays: Vec<String>,
}

impl NostrService {
    pub async fn new(keys_file_path: &PathBuf, relays: &str) -> Result<Self> {
        let (server_key, user_key, sent_info) = match File::open(keys_file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let keys: Self = serde_json::from_reader(reader).context("Failed to parse JSON")?;
                (keys.server_key, keys.user_key, keys.sent_info)
            }
            Err(_) => {
                let (server_key, user_key) = Self::generate_keys()?;
                (server_key, user_key, false)
            }
        };

        let lines = relays.split(',').collect::<Vec<_>>();
        let relays = lines
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let client = Client::new(Keys::new(server_key.into()));
        let service = Self {
            client,
            server_key,
            user_key,
            sent_info,
            keys_file_path: keys_file_path.clone(),
            relays,
        };

        service.add_relays().await?;

        if !sent_info {
            service.write_keys().context("Failed to write keys")?;
        }

        Ok(service)
    }

    pub fn set_sent_info(&mut self, sent_info: bool) {
        self.sent_info = sent_info;
    }

    fn generate_keys() -> Result<(SecretKey, SecretKey)> {
        let server_keys = Keys::generate();
        let server_key = server_keys.secret_key()?;
        let user_keys = Keys::generate();
        let user_key = user_keys.secret_key()?;
        Ok((**server_key, **user_key))
    }

    fn write_keys(&self) -> Result<()> {
        let json_str = serde_json::to_string(&self).context("Failed to serialize data")?;
        let mut file =
            File::create(self.keys_file_path.clone()).context("Failed to create file")?;
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

    async fn add_relays(&self) -> Result<()> {
        for relay in self.relays.iter() {
            self.client.add_relay(relay).await?;
        }
        Ok(())
    }

    pub async fn send_event(&self, event: Event) -> Result<EventId, anyhow::Error> {
        self.client
            .send_event(event)
            .await
            .map_err(|e| anyhow!("Failed to send event: {}", e))
    }

    pub async fn send_encrypted_response(
        &self,
        event: &Event,
        content: Response,
        d_tag: Option<Tag>,
    ) -> Result<(), anyhow::Error> {
        let encrypted = nip04::encrypt(
            self.server_keys().secret_key()?,
            &self.user_keys().public_key(),
            content.as_json(),
        )?;
        let p_tag = Tag::public_key(event.pubkey);
        let e_tag = Tag::event(event.id);
        let tags = match d_tag {
            None => vec![p_tag, e_tag],
            Some(d_tag) => vec![p_tag, e_tag, d_tag],
        };
        let response = EventBuilder::new(Kind::WalletConnectResponse, encrypted, tags)
            .to_event(&self.server_keys())?;

        self.send_event(response).await?;
        Ok(())
    }

    pub async fn broadcast_info_event(&mut self) -> Result<(), anyhow::Error> {
        if self.sent_info {
            info!("Already sent info event");
            return Ok(());
        }
        let content = METHODS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let info = EventBuilder::new(Kind::WalletConnectInfo, content, [])
            .to_event(&self.server_keys())?;
        info!("Broadcasting info event: {}", info.as_json());
        let event_id = self.send_event(info).await?;
        info!("Broadcasted info event: {}", event_id);
        self.sent_info = true;
        self.write_keys()?;
        Ok(())
    }

    pub async fn connect(&self) {
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

    pub async fn subscribe_nwc(&self) {
        let subscription = Filter::new()
            .kinds(vec![Kind::WalletConnectRequest])
            .author(self.user_keys().public_key())
            .pubkey(self.server_keys().public_key())
            .since(Timestamp::now());

        self.client.subscribe(vec![subscription], None).await;

        info!("Listening for nip 47 requests...");
    }

    pub async fn new_nwc_uri(&self) -> Result<NostrWalletConnectURI> {
        let relay = self
            .relays
            .first()
            .ok_or_else(|| anyhow::anyhow!("No relays provided, cannot generate URI"))?;
        let uri = NostrWalletConnectURI::new(
            self.server_keys().public_key(),
            Url::from_str(relay)?,
            self.user_keys().secret_key()?.clone(),
            None,
        );
        Ok(uri)
    }
}
