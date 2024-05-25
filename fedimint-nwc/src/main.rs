use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use multimint::fedimint_core::api::InviteCode;
use tracing::info;

pub mod state;

use clap::{Parser, Subcommand};
use state::AppState;

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Kody Low")]
struct Cli {
    /// Federation invite code
    #[clap(long, env = "FEDIMINT_CLIENTD_INVITE_CODE", required = false)]
    invite_code: String,

    /// Path to FM database
    #[clap(long, env = "FEDIMINT_CLIENTD_DB_PATH", required = true)]
    db_path: PathBuf,

    /// Addr
    #[clap(long, env = "FEDIMINT_CLIENTD_ADDR", required = true)]
    addr: String,

    /// Manual secret
    #[clap(long, env = "FEDIMINT_CLIENTD_MANUAL_SECRET", required = false)]
    manual_secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli: Cli = Cli::parse();

    let mut state = AppState::new(cli.db_path).await?;

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
