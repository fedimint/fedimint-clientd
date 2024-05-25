use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use multimint::fedimint_core::api::InviteCode;
use multimint::MultiMint;

#[derive(Debug, Clone)]
pub struct MultiMintService {
    multimint: MultiMint,
}

impl MultiMintService {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let clients = MultiMint::new(db_path).await?;
        clients.update_gateway_caches().await?;
        Ok(Self { multimint: clients })
    }

    pub async fn init_multimint(
        &mut self,
        invite_code: &str,
        manual_secret: Option<String>,
    ) -> Result<()> {
        match InviteCode::from_str(invite_code) {
            Ok(invite_code) => {
                let federation_id = self
                    .multimint
                    .register_new(invite_code, manual_secret)
                    .await?;
                tracing::info!("Created client for federation id: {:?}", federation_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Invalid federation invite code: {}", e);
                Err(e.into())
            }
        }
    }
}
