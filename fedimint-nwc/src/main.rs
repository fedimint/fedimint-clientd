use anyhow::Result;
use clap::Parser;
use nostr_sdk::{JsonUtil, Kind, RelayPoolNotification};
use tracing::{error, info};

pub mod config;
pub mod database;
pub mod nwc;
pub mod server;
pub mod services;
pub mod state;

use crate::config::Cli;
use crate::server::run_server;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging_and_env()?;
    let cli = Cli::parse();
    let state = AppState::new(cli).await?;

    state.nostr_service.connect().await;
    state.nostr_service.broadcast_info_event().await?;

    let server_handle = tokio::spawn(async {
        match run_server().await {
            Ok(_) => info!("Server ran successfully."),
            Err(e) => {
                error!("Server failed to run: {}", e);
                std::process::exit(1);
            }
        }
    });

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    tokio::select! {
        _ = &mut ctrl_c => {
            info!("Ctrl+C received. Shutting down...");
        },
        _ = event_loop(state.clone()) => {
            info!("Event loop exited unexpectedly.");
        },
        _ = server_handle => {
            info!("Server task exited unexpectedly.");
        }
    }

    shutdown(state).await
}

async fn event_loop(state: AppState) -> Result<()> {
    let mut notifications = state.nostr_service.notifications();
    info!("Listening for events...");
    loop {
        tokio::select! {
            notification = notifications.recv() => {
                if let Ok(notification) = notification {
                    handle_notification(notification, &state).await?;
                }
            }
        }
    }
}

async fn handle_notification(notification: RelayPoolNotification, state: &AppState) -> Result<()> {
    match notification {
        RelayPoolNotification::Event { event, .. } => {
            if event.kind == Kind::WalletConnectRequest
                && event.pubkey == state.nostr_service.user_keys().public_key()
                && event.verify().is_ok()
            {
                info!("Received event: {}", event.as_json());
                state.handle_event(*event).await;
            } else {
                error!("Invalid nwc event: {}", event.as_json());
            }
            Ok(())
        }
        RelayPoolNotification::Shutdown => {
            info!("Relay pool shutdown");
            Err(anyhow::anyhow!("Relay pool shutdown"))
        }
        _ => {
            error!("Unhandled relay pool notification: {notification:?}");
            Ok(())
        }
    }
}

async fn shutdown(state: AppState) -> Result<()> {
    info!("Shutting down services and server...");
    state.wait_for_active_requests().await;
    info!("All active requests completed.");
    state.nostr_service.disconnect().await?;
    info!("Services disconnected.");
    Ok(())
}

fn init_logging_and_env() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    Ok(())
}
