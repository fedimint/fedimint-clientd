pub mod types;

use crate::FedimintClient;
pub use types::*;

impl FedimintClient {
    pub async fn create_deposit_address(
        &self,
        timeout: u64,
    ) -> Result<CreateDepositAddressResponse, String> {
        self.post::<CreateDepositAddressRequest, CreateDepositAddressResponse>(
            "/onchain/deposit-address",
            CreateDepositAddressRequest {
                federationId: self.active_federation_id.to_owned(),
                timeout,
            },
        )
        .await
    }

    pub async fn await_deposit(
        &self,
        operation_id: String,
    ) -> Result<AwaitDepositResponse, String> {
        self.post::<AwaitDepositRequest, AwaitDepositResponse>(
            "/onchain/await-deposit",
            AwaitDepositRequest {
                federationId: self.active_federation_id.to_owned(),
                operationId: operation_id,
            },
        )
        .await
    }
}
