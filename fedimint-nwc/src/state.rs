use std::path::PathBuf;

use anyhow::Result;
use multimint::MultiMint;

#[derive(Debug, Clone)]
pub struct AppState {
    pub multimint: MultiMint,
}

impl AppState {
    pub async fn new(fm_db_path: PathBuf) -> Result<Self> {
        let clients = MultiMint::new(fm_db_path).await?;
        clients.update_gateway_caches().await?;
        Ok(Self { multimint: clients })
    }
}
