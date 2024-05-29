use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use futures_util::StreamExt;
use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription, Description};
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::api::InviteCode;
use multimint::fedimint_core::config::{FederationId, FederationIdPrefix};
use multimint::fedimint_core::core::OperationId;
use multimint::fedimint_core::Amount;
use multimint::fedimint_ln_client::{
    InternalPayState, LightningClientModule, LnPayState, OutgoingLightningPayment, PayType,
};
use multimint::fedimint_ln_common::LightningGateway;
use multimint::MultiMint;
use nostr::nips::nip47::{
    ErrorCode, Method, NIP47Error, PayInvoiceResponseResult, Response, ResponseResult,
};
use nostr::util::hex;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MultiMintService {
    multimint: MultiMint,
    default_federation_id: Option<FederationId>,
}

impl MultiMintService {
    pub async fn new(
        db_path: PathBuf,
        invite_code: InviteCode,
        manual_secret: Option<String>,
    ) -> Result<Self> {
        let mut clients = MultiMint::new(db_path).await?;
        clients
            .register_new(invite_code.clone(), manual_secret.clone())
            .await?;
        clients.update_gateway_caches().await?;
        Ok(Self {
            multimint: clients,
            default_federation_id: Some(invite_code.federation_id()),
        })
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
                Err(e)
            }
        }
    }

    // Helper function to get a specific client from the state or default
    pub async fn get_client(
        &self,
        federation_id: Option<FederationId>,
    ) -> Result<ClientHandleArc, anyhow::Error> {
        let federation_id = match federation_id {
            Some(id) => id,
            None => match self.default_federation_id {
                Some(id) => id,
                None => return Err(anyhow!("No default federation id set")),
            },
        };
        match self.multimint.get(&federation_id).await {
            Some(client) => Ok(client),
            None => Err(anyhow!("No client found for federation id")),
        }
    }

    pub async fn get_client_by_prefix(
        &self,
        federation_id_prefix: &FederationIdPrefix,
    ) -> Result<ClientHandleArc, anyhow::Error> {
        match self.multimint.get_by_prefix(federation_id_prefix).await {
            Some(client) => Ok(client),
            None => Err(anyhow!("No client found for federation id prefix")),
        }
    }

    // Helper method to select a gateway
    async fn get_gateway(&self, client: &ClientHandleArc) -> Result<LightningGateway> {
        let lightning_module = client.get_first_module::<LightningClientModule>();
        let gateways = lightning_module.list_gateways().await;

        let selected_gateway = gateways
            .first()
            .ok_or_else(|| anyhow!("No gateways available"))?
            .info
            .clone();

        Ok(selected_gateway)
    }

    pub async fn pay_invoice(&self, invoice: Bolt11Invoice, method: Method) -> Result<Response> {
        let client = self.get_client(None).await?;
        let gateway = self.get_gateway(&client).await?;
        info!("Paying invoice: {invoice:?}");
        let lightning_module = client.get_first_module::<LightningClientModule>();
        let payment = lightning_module
            .pay_bolt11_invoice(Some(gateway), invoice, ())
            .await?;

        let response = wait_for_ln_payment(&client, payment, false).await?;

        let response = match response {
            Some(ln_response) => {
                info!("Paid invoice: {}", ln_response.contract_id);
                let preimage = hex::encode(ln_response.preimage);
                Response {
                    result_type: method,
                    error: None,
                    result: Some(ResponseResult::PayInvoice(PayInvoiceResponseResult {
                        preimage,
                    })),
                }
            }
            None => {
                let error_msg = "Payment failed".to_string();
                Response {
                    result_type: method,
                    error: Some(NIP47Error {
                        code: ErrorCode::PaymentFailed,
                        message: error_msg,
                    }),
                    result: None,
                }
            }
        };

        Ok(response)
    }

    pub async fn make_invoice(
        &self,
        amount_msat: u64,
        description: String,
        expiry_time: Option<u64>,
    ) -> Result<Bolt11Invoice> {
        let client = self.get_client(None).await?;
        let gateway = self.get_gateway(&client).await?;
        let lightning_module = client.get_first_module::<LightningClientModule>();
        // TODO: spawn invoice subscription to this operation
        let (_, invoice, _) = lightning_module
            .create_bolt11_invoice(
                Amount::from_msats(amount_msat),
                Bolt11InvoiceDescription::Direct(&Description::new(description)?),
                expiry_time,
                (),
                Some(gateway),
            )
            .await?;

        Ok(invoice)
    }
}

#[derive(Debug, Clone)]
pub struct LnPayResponse {
    pub operation_id: OperationId,
    pub payment_type: PayType,
    pub contract_id: String,
    pub fee: Amount,
    pub preimage: String,
}

pub async fn wait_for_ln_payment(
    client: &ClientHandleArc,
    payment: OutgoingLightningPayment,
    return_on_funding: bool,
) -> anyhow::Result<Option<LnPayResponse>> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    match payment.payment_type {
        PayType::Internal(operation_id) => {
            let mut updates = lightning_module
                .subscribe_internal_pay(operation_id)
                .await?
                .into_stream();

            while let Some(update) = updates.next().await {
                match update {
                    InternalPayState::Preimage(preimage) => {
                        return Ok(Some(LnPayResponse {
                            operation_id,
                            payment_type: payment.payment_type,
                            contract_id: payment.contract_id.to_string(),
                            fee: Amount::ZERO,
                            preimage: hex::encode(preimage.0),
                        }));
                    }
                    InternalPayState::RefundSuccess { out_points, error } => {
                        let e = format!(
                            "Internal payment failed. A refund was issued to {:?} Error: {error}",
                            out_points
                        );
                        bail!("{e}");
                    }
                    InternalPayState::UnexpectedError(e) => {
                        bail!("{e}");
                    }
                    InternalPayState::Funding if return_on_funding => return Ok(None),
                    InternalPayState::Funding => {}
                    InternalPayState::RefundError {
                        error_message,
                        error,
                    } => bail!("RefundError: {error_message} {error}"),
                    InternalPayState::FundingFailed { error } => {
                        bail!("FundingFailed: {error}")
                    }
                }
                info!("Update: {update:?}");
            }
        }
        PayType::Lightning(operation_id) => {
            let mut updates = lightning_module
                .subscribe_ln_pay(operation_id)
                .await?
                .into_stream();

            while let Some(update) = updates.next().await {
                let update_clone = update.clone();
                match update_clone {
                    LnPayState::Success { preimage } => {
                        return Ok(Some(LnPayResponse {
                            operation_id,
                            payment_type: payment.payment_type,
                            contract_id: payment.contract_id.to_string(),
                            fee: Amount::ZERO,
                            preimage,
                        }));
                    }
                    LnPayState::Refunded { gateway_error } => {
                        info!("{gateway_error}");
                        Err(anyhow::anyhow!("Payment was refunded"))?;
                    }
                    LnPayState::Canceled => {
                        Err(anyhow::anyhow!("Payment was canceled"))?;
                    }
                    LnPayState::Created
                    | LnPayState::AwaitingChange
                    | LnPayState::WaitingForRefund { .. } => {}
                    LnPayState::Funded if return_on_funding => return Ok(None),
                    LnPayState::Funded => {}
                    LnPayState::UnexpectedError { error_message } => {
                        bail!("UnexpectedError: {error_message}")
                    }
                }
                info!("Update: {update:?}");
            }
        }
    };
    bail!("Lightning Payment failed")
}
