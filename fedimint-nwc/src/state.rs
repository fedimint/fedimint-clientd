use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use multimint::fedimint_core::api::InviteCode;
use nostr_sdk::{Event, EventId, JsonUtil, Kind};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::Cli;
use crate::managers::PaymentsManager;
use crate::nwc::handle_nwc_request;
use crate::services::{MultiMintService, NostrService};

#[derive(Debug, Clone)]
pub struct AppState {
    pub active_requests: Arc<Mutex<BTreeSet<EventId>>>,
    pub multimint_service: MultiMintService,
    pub nostr_service: NostrService,
    pub payments_manager: PaymentsManager,
}

impl AppState {
    pub async fn new(cli: Cli) -> Result<Self, anyhow::Error> {
        let invite_code = InviteCode::from_str(&cli.invite_code)?;
        let multimint_service =
            MultiMintService::new(cli.db_path, Some(invite_code.federation_id())).await?;
        let nostr_service = NostrService::new(&cli.keys_file, &cli.relays).await?;

        let active_requests = Arc::new(Mutex::new(BTreeSet::new()));
        let payments_manager =
            PaymentsManager::new(cli.max_amount, cli.daily_limit, cli.rate_limit_secs);

        Ok(Self {
            active_requests,
            multimint_service,
            nostr_service,
            payments_manager,
        })
    }

    pub async fn init(&mut self, cli: &Cli) -> Result<(), anyhow::Error> {
        self.multimint_service
            .init_multimint(&cli.invite_code, cli.manual_secret.clone())
            .await?;
        Ok(())
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

    /// Adds nwc events to active requests set while waiting for them to
    /// complete so they can finish processing before a shutdown.
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
