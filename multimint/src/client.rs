//! LocalClientBuilder is a builder pattern for adding Fedimint Clients to the
//! multimint

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
use fedimint_client::Client;
use fedimint_core::config::ClientConfig;
use fedimint_core::db::{
    Committable, Database, DatabaseTransaction, IDatabaseTransactionOpsCoreTyped,
};
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_wallet_client::WalletClientInit;
use futures_util::StreamExt;
use rand::thread_rng;
use tracing::info;

use crate::db::{FederationConfig, FederationIdKey, FederationIdKeyPrefix};

#[derive(Debug, Clone)]
pub struct LocalClientBuilder {
    work_dir: PathBuf,
}

impl LocalClientBuilder {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

impl LocalClientBuilder {
    /// Build a new client with the given config and optional manual secret
    #[allow(clippy::too_many_arguments)]
    pub async fn build(
        &self,
        config: FederationConfig,
        manual_secret: Option<[u8; 64]>,
    ) -> Result<fedimint_client::ClientHandleArc> {
        let federation_id = config.invite_code.federation_id();

        let db_path = self.work_dir.join(format!("{federation_id}.db"));

        let db = Database::new(
            fedimint_rocksdb::RocksDb::open(db_path.clone())?,
            Default::default(),
        );

        let mut client_builder = Client::builder(db);
        client_builder.with_module(WalletClientInit(None));
        client_builder.with_module(MintClientInit);
        client_builder.with_module(LightningClientInit);
        client_builder.with_primary_module(1);

        let client_secret =
            match Client::load_decodable_client_secret::<[u8; 64]>(client_builder.db()).await {
                Ok(secret) => secret,
                Err(_) => {
                    if let Some(manual_secret) = manual_secret {
                        info!("Using manual secret provided by user and writing to client storage");
                        Client::store_encodable_client_secret(client_builder.db(), manual_secret)
                            .await?;
                        manual_secret
                    } else {
                        info!("Generating new secret and writing to client storage");
                        let secret = PlainRootSecretStrategy::random(&mut thread_rng());
                        Client::store_encodable_client_secret(client_builder.db(), secret).await?;
                        secret
                    }
                }
            };

        let root_secret = PlainRootSecretStrategy::to_root_secret(&client_secret);
        let client_res = if Client::is_initialized(client_builder.db()).await {
            client_builder.open(root_secret).await
        } else {
            let client_config =
                ClientConfig::download_from_invite_code(&config.invite_code).await?;
            client_builder
                // TODO: make this configurable?
                .join(root_secret, client_config.to_owned())
                .await
        }?;

        Ok(Arc::new(client_res))
    }

    /// Save the federation config to the database
    pub async fn save_config(
        &self,
        config: FederationConfig,
        mut dbtx: DatabaseTransaction<'_, Committable>,
    ) -> Result<()> {
        let id = config.invite_code.federation_id();
        dbtx.insert_entry(&FederationIdKey { id }, &config).await;
        dbtx.commit_tx_result()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save config: {:?}", e))
    }

    pub async fn load_configs(&self, mut dbtx: DatabaseTransaction<'_>) -> Vec<FederationConfig> {
        dbtx.find_by_prefix(&FederationIdKeyPrefix)
            .await
            .collect::<BTreeMap<FederationIdKey, FederationConfig>>()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>()
    }
}
