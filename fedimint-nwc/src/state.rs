use std::collections::BTreeSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use multimint::fedimint_core::api::InviteCode;
use nostr_sdk::{Event, EventId, JsonUtil, Kind};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::Cli;
use crate::database::Database;
use crate::nwc::handle_nwc_request;
use crate::services::{MultiMintService, NostrService};

#[derive(Debug, Clone)]
pub struct AppState {
    pub active_requests: Arc<Mutex<BTreeSet<EventId>>>,
    pub multimint_service: MultiMintService,
    pub nostr_service: NostrService,
    pub db: Database,
}

impl AppState {
    pub async fn new(cli: Cli) -> Result<Self, anyhow::Error> {
        let invite_code = InviteCode::from_str(&cli.invite_code)?;

        // Define paths for MultiMint and Redb databases within the work_dir
        let multimint_db_path = cli.work_dir.join("multimint_db");
        let redb_db_path = cli.work_dir.join("redb_db");
        let keys_file_path = cli.work_dir.join("keys.json");

        // Ensure directories exist
        std::fs::create_dir_all(&multimint_db_path)?;
        std::fs::create_dir_all(&redb_db_path)?;

        let multimint_service =
            MultiMintService::new(multimint_db_path, Some(invite_code.federation_id())).await?;
        let nostr_service = NostrService::new(&keys_file_path, &cli.relays).await?;

        let active_requests = Arc::new(Mutex::new(BTreeSet::new()));
        let db = Database::new(
            &redb_db_path,
            cli.max_amount,
            cli.daily_limit,
            cli.rate_limit_secs,
        )?;

        Ok(Self {
            active_requests,
            multimint_service,
            nostr_service,
            db,
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
