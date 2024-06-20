pub mod types;

use crate::FedimintClient;
pub use types::*;

impl FedimintClient {
    pub async fn create_deposit_address(
        &self,
        timeout: u64,
    ) -> Result<CreateDepositAddressResponse, String> {
        let federation_id = self.active_federation_id.clone();

        if federation_id.is_empty() {
            return Err("Federation ID Required".to_string());
        }

        self.post::<CreateDepositAddressRequest, CreateDepositAddressResponse>(
            "/onchain/deposit-address",
            CreateDepositAddressRequest {
                federationId: federation_id,
                timeout,
            },
        )
        .await
    }

    pub async fn await_deposit(
        &self,
        operation_id: String,
    ) -> Result<AwaitDepositResponse, String> {
        let federation_id = self.active_federation_id.clone();

        if federation_id.is_empty() {
            return Err("Federation ID Required".to_string());
        }

        self.post::<AwaitDepositRequest, AwaitDepositResponse>(
            "/onchain/await-deposit",
            AwaitDepositRequest {
                federationId: federation_id,
                operationId: operation_id,
            },
        )
        .await
    }
}
