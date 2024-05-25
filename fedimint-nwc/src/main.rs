use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use nostr_sdk::RelayPoolNotification;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;
use tracing::{debug, info};

pub mod config;
pub mod nwc;
pub mod state;

use state::AppState;

use crate::config::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let state = state::init(cli).await?;

    // Shutdown signal handler
    let (tx, rx) = oneshot::channel::<()>();
    let signal_handler = tokio::spawn(handle_signals(tx));
    info!("Shutdown signal handler started...");

    // Start the event loop
    event_loop(state.clone()).await?;

    // Wait for shutdown signal
    info!("Server is running. Press CTRL+C to exit.");
    let _ = rx.await;
    info!("Shutting down...");
    state.wait_for_active_requests().await;
    let _ = signal_handler.await;

    Ok(())
}

async fn handle_signals(tx: oneshot::Sender<()>) -> Result<()> {
    let signals = tokio::signal::unix::signal(SignalKind::terminate())
        .or_else(|_| tokio::signal::unix::signal(SignalKind::interrupt()));

    match signals {
        Ok(mut stream) => {
            while stream.recv().await.is_some() {
                debug!("Received shutdown signal");
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to install signal handlers: {}", e));
        }
    }

    let _ = tx.send(());
    Ok(())
}

async fn event_loop(state: AppState) -> Result<()> {
    state.nostr_client.connect().await;
    loop {
        info!("Listening for events...");
        let (tx, _) = tokio::sync::watch::channel(());
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(60 * 15)).await;
            let _ = tx.send(());
        });

        let mut notifications = state.nostr_client.notifications();
        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayPoolNotification::Event { event, .. } => state.handle_event(*event).await,
                RelayPoolNotification::Shutdown => {
                    info!("Relay pool shutdown");
                    break;
                }
                _ => {}
            }
        }

        state.nostr_client.disconnect().await?;
    }
}
