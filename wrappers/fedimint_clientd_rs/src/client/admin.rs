use serde_json::Value;

use crate::{
    options::JoinOptions,
    types::{
        DiscoverVersionRequest, DiscoverVersionResponse, FederationIdsResponse, InfoResponse,
        JoinRequest, JoinResponse, ListOperationsRequest, ListOperationsResponse,
    },
    FedimintClient,
};

impl FedimintClient {
    /// Returns info about each joined federation
    pub async fn info(&self) -> Result<InfoResponse, String> {
        self.get::<InfoResponse>("/admin/info").await
    }

    /// Returns the client configuration
    pub async fn config(&self) -> Result<Value, String> {
        self.get::<Value>("/admin/config").await
    }

    /// Returns the current set of connected federation IDs
    pub async fn federation_ids(&self) -> Result<FederationIdsResponse, String> {
        self.get::<FederationIdsResponse>("/admin/federation-ids")
            .await
    }

    /// Returns the common API version to use to communicate with the federation and modules
    pub async fn discover_version(
        &self,
        threshold: usize,
    ) -> Result<DiscoverVersionResponse, String> {
        self.post::<DiscoverVersionRequest, DiscoverVersionResponse>(
            "/admin/discover-version",
            DiscoverVersionRequest { threshold },
        )
        .await
    }

    /// Output a list of the most recent operations performed by this client on the currently-active federation
    pub async fn list_operations(&self, limit: u64) -> Result<ListOperationsResponse, String> {
        self.post::<ListOperationsRequest, ListOperationsResponse>(
            "/admin/list-operations",
            ListOperationsRequest {
                limit,
                federationId: self.active_federation_id.clone(),
            },
        )
        .await
    }

    pub async fn join(&mut self, options: JoinOptions) -> Result<JoinResponse, String> {
        let response = self
            .post::<JoinRequest, JoinResponse>(
                "/admin/join",
                JoinRequest {
                    inviteCode: options.invite_code,
                    useManualSecret: options.use_manual_secret,
                },
            )
            .await;

        match response {
            Ok(res) => {
                if options.set_active_federation_id {
                    let _ = self
                        .set_active_federation_id(
                            res.clone().this_federation_id,
                            options.use_default_gateway,
                        )
                        .await;
                }

                Ok(res)
            }
            Err(err) => Err(err),
        }
    }
}
