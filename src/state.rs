use std::path::PathBuf;
use anyhow::{Result, anyhow};
use axum::http::StatusCode;
use fedimint_client::ClientArc;
use fedimint_core::config::FederationId;
use multimint::MultiMint;

use crate::error::AppError;
#[derive(Debug, Clone)]
pub struct AppState {
    pub clients: MultiMint,
}

impl AppState {
    pub async fn new(fm_db_path: PathBuf) -> Result<Self> {
        let clients = MultiMint::new(fm_db_path).await?;
        Ok(Self { clients })
    }

    // Helper function to get a specific client from the state or default
    pub async fn get_client(&self, federation_id: Option<FederationId>) -> Result<ClientArc, AppError> {
        let client = match federation_id {
            Some(federation_id) => self.clients.get(&federation_id).await,
            None => self.clients.get_default().await,
        };

        match client {
            Some(client) => Ok(client),
            None => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("No client found for federation id"),
            )),
        }
    }
}
