use anyhow::Result;
use tracing::info;

mod config;
mod error;
mod router;
mod state;
mod types;

mod utils;
use state::{load_fedimint_client, AppState};

use crate::config::CONFIG;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state =
        AppState {
            fm: load_fedimint_client().await?,
        };

    let app = router::create_router(state).await?;

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.host, CONFIG.port))
        .await
        .unwrap();
    info!("Janus Listening on {}", CONFIG.port);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
