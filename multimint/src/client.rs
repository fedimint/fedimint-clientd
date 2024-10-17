//! LocalClientBuilder is a builder pattern for adding Fedimint Clients to the
//! multimint

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use bip39::Mnemonic;
use fedimint_bip39::Bip39RootSecretStrategy;
use fedimint_client::db::ClientConfigKey;
use fedimint_client::derivable_secret::{ChildId, DerivableSecret};
use fedimint_client::module::init::ClientModuleInitRegistry;
use fedimint_client::secret::RootSecretStrategy;
use fedimint_client::{Client, ClientBuilder};
use fedimint_core::config::FederationId;
use fedimint_core::db::{
    Committable, Database, DatabaseTransaction, IDatabaseTransactionOpsCoreTyped,
};
use fedimint_core::encoding::Encodable;
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_wallet_client::WalletClientInit;
use futures_util::StreamExt;

use crate::db::{FederationConfig, FederationIdKey, FederationIdKeyPrefix};

#[derive(Debug, Clone)]
pub struct LocalClientBuilder {
    mnemonic: Mnemonic,
}

impl LocalClientBuilder {
    pub fn new(mnemonic: Mnemonic) -> Self {
        Self { mnemonic }
    }
}

impl LocalClientBuilder {
    /// Build a new client with the given config and optional manual secret
    #[allow(clippy::too_many_arguments)]
    pub async fn build(
        &self,
        db: &Database,
        config: FederationConfig,
    ) -> Result<fedimint_client::ClientHandleArc> {
        let federation_id = config.invite_code.federation_id();
        let db = db.with_prefix(federation_id.consensus_encode_to_vec());
        let secret = self.derive_federation_secret(&federation_id);
        Self::verify_client_config(&db, federation_id).await?;

        let client_builder = self.create_client_builder(db.clone()).await?;

        let client_res = if Client::is_initialized(&db).await {
            client_builder.open(secret).await
        } else {
            let client_config =
                fedimint_api_client::download_from_invite_code(&config.invite_code).await?;
            client_builder
                .join(secret, client_config.to_owned(), None)
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

    pub fn derive_federation_secret(&self, federation_id: &FederationId) -> DerivableSecret {
        let global_root_secret = Bip39RootSecretStrategy::<12>::to_root_secret(&self.mnemonic);
        let multi_federation_root_secret = global_root_secret.child_key(ChildId(0));
        let federation_root_secret = multi_federation_root_secret.federation_key(federation_id);
        let federation_wallet_root_secret = federation_root_secret.child_key(ChildId(0));
        federation_wallet_root_secret.child_key(ChildId(0))
    }

    /// Verifies that the saved `ClientConfig` contains the expected
    /// federation's config.
    async fn verify_client_config(db: &Database, federation_id: FederationId) -> Result<()> {
        let mut dbtx = db.begin_transaction_nc().await;
        if let Some(config) = dbtx.get_value(&ClientConfigKey).await {
            if config.calculate_federation_id() != federation_id {
                anyhow::bail!("Federation Id did not match saved federation ID")
            }
        }
        Ok(())
    }

    /// Constructs the client builder with the modules, database, and connector
    /// used to create clients for connected federations.
    async fn create_client_builder(&self, db: Database) -> Result<ClientBuilder> {
        let mut registry = ClientModuleInitRegistry::new();
        registry.attach(WalletClientInit::default());
        registry.attach(MintClientInit);
        registry.attach(LightningClientInit::default());
        let mut client_builder = Client::builder(db).await?;
        client_builder.with_module_inits(registry);
        client_builder.with_primary_module(1);
        Ok(client_builder)
    }
}
