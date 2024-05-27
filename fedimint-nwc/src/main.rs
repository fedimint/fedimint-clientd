use anyhow::Result;
use clap::Parser;
use nostr_sdk::{JsonUtil, Kind, RelayPoolNotification};
use tokio::{pin, task};
use tracing::{error, info};

pub mod config;
pub mod database;
pub mod nwc;
pub mod services;

pub mod state;
use axum::Router;
use state::AppState;
use tower_http::services::ServeDir;

use crate::config::Cli;

/// Fedimint Nostr Wallet Connect
/// A nostr wallet connect implementation on top of a multimint client
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli = Cli::parse();
    let state = AppState::new(cli).await?;

    // Connect to the relay pool and broadcast the nwc info event on startup
    state.nostr_service.connect().await;
    state.nostr_service.broadcast_info_event().await?;

    let server = Router::new().nest_service("/", ServeDir::new("frontend/assets"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Spawn a new Tokio task for the server
    let server_task = task::spawn(async move {
        axum::serve(listener, server).await.unwrap();
    });

    // Wait for the server task to complete if necessary
    server_task.await?;

    // Start the event loop
    event_loop(state.clone()).await?;

    Ok(())
}

/// Event loop that listens for nostr wallet connect events and handles them
async fn event_loop(state: AppState) -> Result<()> {
    // Handle ctrl+c to gracefully shutdown the event loop
    let ctrl_c = tokio::signal::ctrl_c();
    pin!(ctrl_c);

    let mut notifications = state.nostr_service.notifications();
    info!("Listening for events...");
    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                info!("Ctrl+C received. Waiting for active requests to complete...");
                state.wait_for_active_requests().await;
                info!("All active requests completed.");
                break;
            },
            notification = notifications.recv() => {
                if let Ok(notification) = notification {
                    if let RelayPoolNotification::Event { event, .. } = notification {
                        // Only handle nwc events
                        if event.kind == Kind::WalletConnectRequest
                            && event.pubkey == state.nostr_service.user_keys().public_key()
                            && event.verify().is_ok() {
                                info!("Received event: {}", event.as_json());
                                state.handle_event(*event).await
                        } else {
                            error!("Invalid nwc event: {}", event.as_json());
                        }
                    } else if let RelayPoolNotification::Shutdown = notification {
                        info!("Relay pool shutdown");
                        break;
                    } else {
                        error!("Unhandled relay pool notification: {notification:?}");
                    }
                }
            }
        }
    }

    state.nostr_service.disconnect().await?;
    Ok(())
}
