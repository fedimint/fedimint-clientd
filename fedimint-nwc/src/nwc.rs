use std::str::FromStr;

use anyhow::{anyhow, Result};
use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription};
use nostr::nips::nip04;
use nostr::nips::nip47::{
    ErrorCode, LookupInvoiceResponseResult, MakeInvoiceRequestParams, Method, NIP47Error,
    PayInvoiceRequestParams, PayKeysendRequestParams, Request, RequestParams, Response,
    ResponseResult,
};
use nostr::util::hex;
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
            let mut pm = state.payments_manager.clone();
            handle_nwc_params(
                params,
                req.method,
                &event,
                &state.multimint_service,
                &state.nostr_service,
                &mut pm,
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
        let mut pm = state.payments_manager.clone();
        spawn(async move {
            handle_nwc_params(params, method, &event_clone, &mm, &nostr, &mut pm).await
        })
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
    pm: &mut PaymentsManager,
) -> Result<(), anyhow::Error> {
    let mut d_tag: Option<Tag> = None;
    let content = match params {
        RequestParams::PayInvoice(params) => {
            handle_pay_invoice(params, method, multimint, pm).await
        }
        RequestParams::PayKeysend(params) => handle_pay_keysend(params, method, pm).await,
        RequestParams::MakeInvoice(params) => {
            handle_make_invoice(params, method, multimint, pm).await
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

    nostr
        .send_encrypted_response(&event, content, d_tag)
        .await?;

    Ok(())
}

async fn handle_pay_invoice(
    params: PayInvoiceRequestParams,
    method: Method,
    multimint: &MultiMintService,
    pm: &mut PaymentsManager,
) -> Response {
    let invoice = match Bolt11Invoice::from_str(&params.invoice)
        .map_err(|_| anyhow!("Failed to parse invoice"))
    {
        Ok(invoice) => invoice,
        Err(e) => {
            error!("Error parsing invoice: {e}");
            return Response {
                result_type: method,
                error: Some(NIP47Error {
                    code: ErrorCode::PaymentFailed,
                    message: format!("Failed to parse invoice: {e}"),
                }),
                result: None,
            };
        }
    };

    let msats = invoice
        .amount_milli_satoshis()
        .or(params.amount)
        .unwrap_or(0);
    let dest = match invoice.payee_pub_key() {
        Some(dest) => dest.to_string(),
        None => "".to_string(), /* FIXME: this is a hack, should handle
                                 * no pubkey case better */
    };

    let error_msg = pm.check_payment_limits(msats, dest.clone());

    // verify amount, convert to msats
    match error_msg {
        None => {
            match multimint.pay_invoice(invoice, method).await {
                Ok(content) => {
                    // add payment to tracker
                    pm.add_payment(msats, dest);
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

async fn handle_pay_keysend(
    params: PayKeysendRequestParams,
    method: Method,
    pm: &mut PaymentsManager,
) -> Response {
    let d_tag = params.id.map(|id| Tag::identifier(id.clone()));
    let msats = params.amount;
    let dest = params.pubkey.clone();

    let error_msg = pm.check_payment_limits(msats, dest);

    match error_msg {
        None => {
            error!("Error paying keysend: UNSUPPORTED IN IMPLEMENTATION");
            Response {
                result_type: method,
                error: Some(NIP47Error {
                    code: ErrorCode::PaymentFailed,
                    message: "Failed to pay keysend: UNSUPPORTED IN IMPLEMENTATION".to_string(),
                }),
                result: None,
            }
        }
        Some(err_msg) => Response {
            result_type: method,
            error: Some(NIP47Error {
                code: ErrorCode::QuotaExceeded,
                message: err_msg,
            }),
            result: None,
        },
    }
}

async fn handle_make_invoice(
    params: MakeInvoiceRequestParams,
    method: Method,
    multimint: &MultiMintService,
    pm: &mut PaymentsManager,
) -> Response {
    let description = match params.description {
        None => "".to_string(),
        Some(desc) => desc,
    };
    let res = multimint
        .make_invoice(params.amount, description, params.expiry)
        .await;
    match res {
        Ok(res) => res,
        Err(e) => Response {
            result_type: Method::MakeInvoice,
            error: Some(NIP47Error {
                code: ErrorCode::PaymentFailed,
                message: format!("Failed to make invoice: {e}"),
            }),
            result: None,
        },
    }
}
