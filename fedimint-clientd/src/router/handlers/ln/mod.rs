use std::str::FromStr;

use anyhow::{bail, Context};
use futures_util::StreamExt;
use lightning_invoice::Bolt11Invoice;
use multimint::fedimint_client::ClientHandleArc;
use multimint::fedimint_core::Amount;
use multimint::fedimint_ln_client::{InternalPayState, LightningClientModule, LnPayState, PayType};
use tracing::{debug, info};

use self::pay::{LnPayRequest, LnPayResponse};

pub mod await_invoice;
pub mod claim_external_receive_tweaked;
pub mod invoice;
pub mod invoice_external_pubkey_tweaked;
pub mod list_gateways;
pub mod pay;

pub async fn get_invoice(req: &LnPayRequest) -> anyhow::Result<Bolt11Invoice> {
    let info = req.payment_info.trim();
    match Bolt11Invoice::from_str(info) {
        Ok(invoice) => {
            debug!("Parsed parameter as bolt11 invoice: {invoice}");
            match (invoice.amount_milli_satoshis(), req.amount_msat) {
                (Some(_), Some(_)) => {
                    bail!("Amount specified in both invoice and command line")
                }
                (None, _) => {
                    bail!("We don't support invoices without an amount")
                }
                _ => {}
            };
            Ok(invoice)
        }
        Err(e) => {
            let lnurl = if info.to_lowercase().starts_with("lnurl") {
                lnurl::lnurl::LnUrl::from_str(info)?
            } else if info.contains('@') {
                lnurl::lightning_address::LightningAddress::from_str(info)?.lnurl()
            } else {
                bail!("Invalid invoice or lnurl: {e:?}");
            };
            debug!("Parsed parameter as lnurl: {lnurl:?}");
            let amount = req
                .amount_msat
                .context("When using a lnurl, an amount must be specified")?;
            let async_client = lnurl::AsyncClient::from_client(reqwest::Client::new());
            let response = async_client.make_request(&lnurl.url).await?;
            match response {
                lnurl::LnUrlResponse::LnUrlPayResponse(response) => {
                    let invoice = async_client
                        .get_invoice(&response, amount.msats, None, req.lnurl_comment.as_deref())
                        .await?;
                    let invoice = Bolt11Invoice::from_str(invoice.invoice())?;
                    assert_eq!(invoice.amount_milli_satoshis(), Some(amount.msats));
                    Ok(invoice)
                }
                other => {
                    bail!("Unexpected response from lnurl: {other:?}");
                }
            }
        }
    }
}

pub async fn wait_for_ln_payment(
    client: &ClientHandleArc,
    payment_type: PayType,
    contract_id: String,
    return_on_funding: bool,
) -> anyhow::Result<Option<LnPayResponse>> {
    let lightning_module = client.get_first_module::<LightningClientModule>();
    match payment_type {
        PayType::Internal(operation_id) => {
            let mut updates = lightning_module
                .subscribe_internal_pay(operation_id)
                .await?
                .into_stream();

            while let Some(update) = updates.next().await {
                match update {
                    InternalPayState::Preimage(_preimage) => {
                        return Ok(Some(LnPayResponse {
                            operation_id,
                            payment_type,
                            contract_id,
                            fee: Amount::ZERO,
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
                    LnPayState::Success { preimage: _ } => {
                        return Ok(Some(LnPayResponse {
                            operation_id,
                            payment_type,
                            contract_id,
                            fee: Amount::ZERO,
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
