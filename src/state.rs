use std::path::PathBuf;

use anyhow::{anyhow, Result};
use axum::http::StatusCode;
use fedimint_client::ClientArc;
use fedimint_core::config::{FederationId, FederationIdPrefix};
use multimint::MultiMint;

use crate::error::AppError;
#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: MultiMint,
}

impl AppState {
    pub async fn new(fm_db_path: PathBuf) -> Result<Self> {
        let clients = MultiMint::new(fm_db_path).await?;
        Ok(Self { multimint: clients })
    }

    // Helper function to get a specific client from the state or default
    pub async fn get_client(
        &self,
        federation_id: Option<FederationId>,
    ) -> Result<ClientArc, AppError> {
        let client = match federation_id {
            Some(federation_id) => self.multimint.get(&federation_id).await,
            None => self.multimint.get_default().await,
        };

        match client {
            Some(client) => Ok(client),
            None => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("No client found for federation id"),
            )),
        }
    }

    pub async fn get_client_by_prefix(
        &self,
        federation_id_prefix: &FederationIdPrefix,
    ) -> Result<ClientArc, AppError> {
        let client = self.multimint.get_by_prefix(federation_id_prefix).await;

        match client {
            Some(client) => Ok(client),
            None => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("No client found for federation id prefix"),
            )),
        }
    }
}
