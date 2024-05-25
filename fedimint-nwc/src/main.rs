use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use config::Cli;
use multimint::fedimint_core::api::InviteCode;
use tracing::info;

pub mod config;
pub mod nwc;
pub mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli: Cli = Cli::parse();

    let mut state = AppState::new(cli.db_path, &cli.keys_file).await?;

    let manual_secret = match cli.manual_secret {
        Some(secret) => Some(secret),
        None => match std::env::var("FEDIMINT_CLIENTD_MANUAL_SECRET") {
            Ok(secret) => Some(secret),
            Err(_) => None,
        },
    };

    match InviteCode::from_str(&cli.invite_code) {
        Ok(invite_code) => {
            let federation_id = state
                .multimint
                .register_new(invite_code, manual_secret)
                .await?;
            info!("Created client for federation id: {:?}", federation_id);
        }
        Err(e) => {
            info!(
                "No federation invite code provided, skipping client creation: {}",
                e
            );
        }
    }

    if state.multimint.all().await.is_empty() {
        return Err(anyhow::anyhow!("No clients found, must have at least one client to start the server. Try providing a federation invite code with the `--invite-code` flag or setting the `FEDIMINT_CLIENTD_INVITE_CODE` environment variable."));
    }

    Ok(())
}
