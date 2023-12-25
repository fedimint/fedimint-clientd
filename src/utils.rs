use std::str::FromStr;

use anyhow::bail;
use fedimint_client::ClientArc;
use fedimint_core::Amount;
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;
use lightning_invoice::Bolt11Invoice;
use tracing::debug;

use crate::types::fedimint::InfoResponse;

pub async fn get_note_summary(client: &ClientArc) -> anyhow::Result<InfoResponse> {
    let mint_client = client.get_first_module::<MintClientModule>();
    let wallet_client = client.get_first_module::<WalletClientModule>();
    let summary =
        mint_client
            .get_wallet_summary(
                &mut client
                    .db()
                    .begin_transaction_nc()
                    .await
                    .to_ref_with_prefix_module_id(1),
            )
            .await;
    Ok(InfoResponse {
        federation_id: client.federation_id(),
        network: wallet_client.get_network().to_string(),
        meta: client.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    })
}

async fn get_invoice(
    info: &str,
    amount: Option<Amount>,
    lnurl_comment: Option<String>,
) -> anyhow::Result<Bolt11Invoice> {
    let info = info.trim();
    match lightning_invoice::Bolt11Invoice::from_str(info) {
        Ok(invoice) => {
            debug!("Parsed parameter as bolt11 invoice: {invoice}");
            match (invoice.amount_milli_satoshis(), amount) {
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
            let amount = amount.context("When using a lnurl, an amount must be specified")?;
            let async_client = lnurl::AsyncClient::from_client(reqwest::Client::new());
            let response = async_client.make_request(&lnurl.url).await?;
            match response {
                lnurl::LnUrlResponse::LnUrlPayResponse(response) => {
                    let invoice = async_client
                        .get_invoice(&response, amount.msats, None, lnurl_comment.as_deref())
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
