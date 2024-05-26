use std::str::FromStr;

use anyhow::{anyhow, Result};
use lightning_invoice::Bolt11Invoice;
use nostr::nips::nip04;
use nostr::nips::nip47::{ErrorCode, Method, NIP47Error, Request, RequestParams, Response};
use nostr::Tag;
use nostr_sdk::{Event, JsonUtil};
use tokio::spawn;
use tracing::{error, info};

use crate::managers::PaymentsManager;
use crate::services::{MultiMintService, NostrService};
use crate::state::AppState;

pub const METHODS: [Method; 8] = [
    Method::GetInfo,
    Method::MakeInvoice,
    Method::GetBalance,
    Method::LookupInvoice,
    Method::PayInvoice,
    Method::MultiPayInvoice,
    Method::PayKeysend,
    Method::MultiPayKeysend,
];

#[derive(Debug, Clone)]
pub struct NwcConfig {
    pub max_amount: u64,
    pub daily_limit: u64,
}

pub async fn handle_nwc_request(state: &AppState, event: Event) -> Result<(), anyhow::Error> {
    let user_keys = state.nostr_service.user_keys();
    let decrypted = nip04::decrypt(user_keys.secret_key()?, &event.pubkey, &event.content)?;
    let req: Request = Request::from_json(&decrypted)?;

    info!("Request params: {:?}", req.params);

    match req.params {
        RequestParams::MultiPayInvoice(params) => {
            handle_multiple_payments(
                params.invoices,
                req.method,
                &event,
                state,
                RequestParams::PayInvoice,
            )
            .await
        }
        RequestParams::MultiPayKeysend(params) => {
            handle_multiple_payments(
                params.keysends,
                req.method,
                &event,
                state,
                RequestParams::PayKeysend,
            )
            .await
        }
        params => {
            handle_nwc_params(
                params,
                req.method,
                &event,
                &state.multimint_service,
                &state.nostr_service,
                &state.payments_manager,
            )
            .await
        }
    }
}

async fn handle_multiple_payments<T>(
    items: Vec<T>,
    method: Method,
    event: &Event,
    state: &AppState,
    param_constructor: fn(T) -> RequestParams,
) -> Result<(), anyhow::Error> {
    for item in items {
        let params = param_constructor(item);
        let event_clone = event.clone();
        let mm = state.multimint_service.clone();
        let nostr = state.nostr_service.clone();
        let pm = state.payments_manager.clone();
        spawn(
            async move { handle_nwc_params(params, method, &event_clone, &mm, &nostr, &pm).await },
        )
        .await??;
    }
    Ok(())
}

async fn handle_nwc_params(
    params: RequestParams,
    method: Method,
    event: &Event,
    multimint: &MultiMintService,
    nostr: &NostrService,
    pm: &PaymentsManager,
) -> Result<(), anyhow::Error> {
    let mut d_tag: Option<Tag> = None;
    let content = match params {
        RequestParams::PayInvoice(params) => {
            d_tag = params.id.map(|id| Tag::identifier(id.clone()));

            let invoice = Bolt11Invoice::from_str(&params.invoice)
                .map_err(|_| anyhow!("Failed to parse invoice"))?;
            let msats = invoice
                .amount_milli_satoshis()
                .or(params.amount)
                .unwrap_or(0);

            let error_msg = pm.check_payment_limits(msats);

            // verify amount, convert to msats
            match error_msg {
                None => {
                    match multimint.pay_invoice(invoice, method).await {
                        Ok(content) => {
                            // add payment to tracker
                            pm.add_payment(msats);
                            content
                        }
                        Err(e) => {
                            error!("Error paying invoice: {e}");

                            Response {
                                result_type: method,
                                error: Some(NIP47Error {
                                    code: ErrorCode::InsufficientBalance,
                                    message: format!("Failed to pay invoice: {e}"),
                                }),
                                result: None,
                            }
                        }
                    }
                }
                Some(err_msg) => Response {
                    result_type: method,
                    error: Some(NIP47Error {
                        code: ErrorCode::QuotaExceeded,
                        message: err_msg.to_string(),
                    }),
                    result: None,
                },
            }
        }
        RequestParams::PayKeysend(params) => {
            d_tag = params.id.map(Tag::Identifier);

            let msats = params.amount;
            let error_msg = if config.max_amount > 0 && msats > config.max_amount * 1_000 {
                Some("Invoice amount too high.")
            } else if config.daily_limit > 0
                && tracker.lock().await.sum_payments() + msats > config.daily_limit * 1_000
            {
                Some("Daily limit exceeded.")
            } else {
                None
            };

            // verify amount, convert to msats
            match error_msg {
                None => {
                    let pubkey = bitcoin::secp256k1::PublicKey::from_str(&params.pubkey)?;
                    match pay_keysend(
                        pubkey,
                        params.preimage,
                        params.tlv_records,
                        msats,
                        lnd,
                        method,
                    )
                    .await
                    {
                        Ok(content) => {
                            // add payment to tracker
                            tracker.lock().await.add_payment(msats);
                            content
                        }
                        Err(e) => {
                            error!("Error paying keysend: {e}");

                            Response {
                                result_type: method,
                                error: Some(NIP47Error {
                                    code: ErrorCode::PaymentFailed,
                                    message: format!("Failed to pay keysend: {e}"),
                                }),
                                result: None,
                            }
                        }
                    }
                }
                Some(err_msg) => Response {
                    result_type: method,
                    error: Some(NIP47Error {
                        code: ErrorCode::QuotaExceeded,
                        message: err_msg.to_string(),
                    }),
                    result: None,
                },
            }
        }
        RequestParams::MakeInvoice(params) => {
            let description_hash: Vec<u8> = match params.description_hash {
                None => vec![],
                Some(str) => FromHex::from_hex(&str)?,
            };
            let inv = Invoice {
                memo: params.description.unwrap_or_default(),
                description_hash,
                value_msat: params.amount as i64,
                expiry: params.expiry.unwrap_or(86_400) as i64,
                private: config.route_hints,
                ..Default::default()
            };
            let res = lnd.add_invoice(inv).await?.into_inner();

            info!("Created invoice: {}", res.payment_request);

            Response {
                result_type: Method::MakeInvoice,
                error: None,
                result: Some(ResponseResult::MakeInvoice(MakeInvoiceResponseResult {
                    invoice: res.payment_request,
                    payment_hash: ::hex::encode(res.r_hash),
                })),
            }
        }
        RequestParams::LookupInvoice(params) => {
            let mut invoice: Option<Bolt11Invoice> = None;
            let payment_hash: Vec<u8> = match params.payment_hash {
                None => match params.invoice {
                    None => return Err(anyhow!("Missing payment_hash or invoice")),
                    Some(bolt11) => {
                        let inv = Bolt11Invoice::from_str(&bolt11)
                            .map_err(|_| anyhow!("Failed to parse invoice"))?;
                        invoice = Some(inv.clone());
                        inv.payment_hash().into_32().to_vec()
                    }
                },
                Some(str) => FromHex::from_hex(&str)?,
            };

            let res = lnd
                .lookup_invoice(PaymentHash {
                    r_hash: payment_hash.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner();

            info!("Looked up invoice: {}", res.payment_request);

            let (description, description_hash) = match invoice {
                Some(inv) => match inv.description() {
                    Bolt11InvoiceDescription::Direct(desc) => (Some(desc.to_string()), None),
                    Bolt11InvoiceDescription::Hash(hash) => (None, Some(hash.0.to_string())),
                },
                None => (None, None),
            };

            let preimage = if res.r_preimage.is_empty() {
                None
            } else {
                Some(hex::encode(res.r_preimage))
            };

            let settled_at = if res.settle_date == 0 {
                None
            } else {
                Some(res.settle_date as u64)
            };

            Response {
                result_type: Method::LookupInvoice,
                error: None,
                result: Some(ResponseResult::LookupInvoice(LookupInvoiceResponseResult {
                    transaction_type: None,
                    invoice: Some(res.payment_request),
                    description,
                    description_hash,
                    preimage,
                    payment_hash: hex::encode(payment_hash),
                    amount: res.value_msat as u64,
                    fees_paid: 0,
                    created_at: res.creation_date as u64,
                    expires_at: (res.creation_date + res.expiry) as u64,
                    settled_at,
                    metadata: Default::default(),
                })),
            }
        }
        RequestParams::GetBalance => {
            let tracker = tracker.lock().await.sum_payments();
            let remaining_msats = config.daily_limit * 1_000 - tracker;
            info!("Current balance: {remaining_msats}msats");
            Response {
                result_type: Method::GetBalance,
                error: None,
                result: Some(ResponseResult::GetBalance(GetBalanceResponseResult {
                    balance: remaining_msats,
                })),
            }
        }
        RequestParams::GetInfo => {
            let lnd_info: GetInfoResponse = lnd.get_info(GetInfoRequest {}).await?.into_inner();
            info!("Getting info");
            Response {
                result_type: Method::GetBalance,
                error: None,
                result: Some(ResponseResult::GetInfo(GetInfoResponseResult {
                    alias: lnd_info.alias,
                    color: lnd_info.color,
                    pubkey: lnd_info.identity_pubkey,
                    network: "".to_string(),
                    block_height: lnd_info.block_height,
                    block_hash: lnd_info.block_hash,
                    methods: METHODS.iter().map(|i| i.to_string()).collect(),
                })),
            }
        }
        _ => {
            return Err(anyhow!("Command not supported"));
        }
    };

    let encrypted = nip04::encrypt(
        &keys.server_key.into(),
        &keys.user_keys().public_key(),
        content.as_json(),
    )?;
    let p_tag = Tag::public_key(event.pubkey);
    let e_tag = Tag::event(event.id);
    let tags = match d_tag {
        None => vec![p_tag, e_tag],
        Some(d_tag) => vec![p_tag, e_tag, d_tag],
    };
    let response = EventBuilder::new(Kind::WalletConnectResponse, encrypted, tags)
        .to_event(&keys.server_keys())?;

    client.send_event(response).await?;

    Ok(())
}
