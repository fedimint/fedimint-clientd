use std::str::FromStr;

use anyhow::{anyhow, Result};
use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription};
use nostr::nips::nip04;
use nostr::nips::nip47::{
    ErrorCode, GetBalanceResponseResult, LookupInvoiceRequestParams, LookupInvoiceResponseResult,
    MakeInvoiceRequestParams, MakeInvoiceResponseResult, Method, NIP47Error,
    PayInvoiceRequestParams, PayKeysendRequestParams, Request, RequestParams, Response,
    ResponseResult,
};
use nostr::util::hex;
use nostr::Tag;
use nostr_sdk::{Event, JsonUtil};
use tokio::spawn;
use tracing::info;

use crate::database::Database;
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
                &state.db,
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
        let mut db = state.db.clone();
        spawn(async move {
            handle_nwc_params(params, method, &event_clone, &mm, &nostr, &mut db).await
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
    db: &Database,
) -> Result<(), anyhow::Error> {
    let d_tag: Option<Tag> = None;
    let response_result = match params {
        RequestParams::PayInvoice(params) => {
            handle_pay_invoice(params, method, multimint, db).await
        }
        RequestParams::PayKeysend(params) => handle_pay_keysend(params, method, db).await,
        RequestParams::MakeInvoice(params) => handle_make_invoice(params, multimint, db).await,
        RequestParams::LookupInvoice(params) => handle_lookup_invoice(params, method, db).await,
        RequestParams::GetBalance => handle_get_balance(method, db).await,
        RequestParams::GetInfo => Err(NIP47Error {
            code: ErrorCode::Unauthorized,
            message: "GetInfo functionality is not implemented yet.".to_string(),
        }),
        _ => {
            return Err(anyhow!("Command not supported"));
        }
    };

    match response_result {
        Ok(response) => nostr.send_encrypted_response(&event, response, d_tag).await,
        Err(e) => {
            let error_response = Response {
                result_type: method,
                error: Some(e),
                result: None,
            };
            nostr
                .send_encrypted_response(&event, error_response, d_tag)
                .await
        }
    }
}

async fn handle_pay_invoice(
    params: PayInvoiceRequestParams,
    method: Method,
    multimint: &MultiMintService,
    db: &Database,
) -> Result<Response, NIP47Error> {
    let invoice = Bolt11Invoice::from_str(&params.invoice).map_err(|e| NIP47Error {
        code: ErrorCode::PaymentFailed,
        message: format!("Failed to parse invoice: {e}"),
    })?;

    let msats = invoice
        .amount_milli_satoshis()
        .or(params.amount)
        .unwrap_or(0);

    db.check_payment_limits(msats).map_err(|err| NIP47Error {
        code: ErrorCode::QuotaExceeded,
        message: err.to_string(),
    })?;

    let response = multimint
        .pay_invoice(invoice.clone(), method)
        .await
        .map_err(|e| NIP47Error {
            code: ErrorCode::InsufficientBalance,
            message: format!("Failed to pay invoice: {e}"),
        })?;

    db.add_payment(invoice).map_err(|e| NIP47Error {
        code: ErrorCode::Unauthorized,
        message: format!("Failed to add payment to tracker: {e}"),
    })?;

    Ok(response)
}

async fn handle_pay_keysend(
    params: PayKeysendRequestParams,
    _method: Method,
    db: &Database,
) -> Result<Response, NIP47Error> {
    let msats = params.amount;

    db.check_payment_limits(msats).map_err(|err| NIP47Error {
        code: ErrorCode::QuotaExceeded,
        message: err.to_string(),
    })?;

    Err(NIP47Error {
        code: ErrorCode::PaymentFailed,
        message: "Failed to pay keysend: UNSUPPORTED IN IMPLEMENTATION".to_string(),
    })
}

async fn handle_make_invoice(
    params: MakeInvoiceRequestParams,
    multimint: &MultiMintService,
    db: &Database,
) -> Result<Response, NIP47Error> {
    let description = params.description.unwrap_or_default();
    let invoice = multimint
        .make_invoice(params.amount, description, params.expiry)
        .await
        .map_err(|e| NIP47Error {
            code: ErrorCode::PaymentFailed,
            message: format!("Failed to make invoice: {e}"),
        })?;

    db.add_invoice(&invoice).map_err(|e| NIP47Error {
        code: ErrorCode::Unauthorized,
        message: format!("Failed to add invoice to database: {e}"),
    })?;

    Ok(Response {
        result_type: Method::MakeInvoice,
        error: None,
        result: Some(ResponseResult::MakeInvoice(MakeInvoiceResponseResult {
            invoice: invoice.to_string(),
            payment_hash: hex::encode(invoice.payment_hash()),
        })),
    })
}

async fn handle_lookup_invoice(
    params: LookupInvoiceRequestParams,
    method: Method,
    db: &Database,
) -> Result<Response, NIP47Error> {
    let invoice = db.lookup_invoice(params).map_err(|e| NIP47Error {
        code: ErrorCode::Unauthorized,
        message: format!("Failed to lookup invoice: {e}"),
    })?;
    let payment_hash = invoice.payment_hash();

    info!("Looked up invoice: {}", payment_hash);

    let (description, description_hash) = match invoice.description() {
        Some(Bolt11InvoiceDescription::Direct(desc)) => (Some(desc.to_string()), None),
        Some(Bolt11InvoiceDescription::Hash(hash)) => (None, Some(hash.0.to_string())),
        None => (None, None),
    };

    let preimage = match invoice.clone().preimage {
        Some(preimage) => Some(hex::encode(preimage)),
        None => None,
    };

    let settled_at = invoice.settled_at();
    let created_at = invoice.created_at();
    let expires_at = invoice.expires_at();
    let invoice_str = invoice.invoice.to_string();
    let amount = invoice.invoice.amount_milli_satoshis().unwrap_or(0) as u64;

    Ok(Response {
        result_type: method,
        error: None,
        result: Some(ResponseResult::LookupInvoice(LookupInvoiceResponseResult {
            transaction_type: None,
            invoice: Some(invoice_str),
            description,
            description_hash,
            preimage,
            payment_hash,
            amount: amount,
            fees_paid: 0,
            created_at,
            expires_at,
            settled_at,
            metadata: Default::default(),
        })),
    })
}

async fn handle_get_balance(method: Method, db: &Database) -> Result<Response, NIP47Error> {
    let tracker = db.sum_payments().map_err(|e| NIP47Error {
        code: ErrorCode::Unauthorized,
        message: format!("Failed to get balance: {e}"),
    })?;
    let remaining_msats = db.daily_limit * 1_000 - tracker;
    info!("Current balance: {remaining_msats}msats");
    Ok(Response {
        result_type: method,
        error: None,
        result: Some(ResponseResult::GetBalance(GetBalanceResponseResult {
            balance: remaining_msats,
        })),
    })
}

// async fn handle_get_info(method: Method, nostr: &NostrService) -> Response {
//     let lnd_info: GetInfoResponse = lnd.get_info(GetInfoRequest
// {}).await?.into_inner();     info!("Getting info");
//     Response {
//         result_type: Method::GetInfo,
//         error: None,
//         result: Some(ResponseResult::GetInfo(GetInfoResponseResult {
//             alias: lnd_info.alias,
//             color: lnd_info.color,
//             pubkey: lnd_info.identity_pubkey,
//             network: "".to_string(),
//             block_height: lnd_info.block_height,
//             block_hash: lnd_info.block_hash,
//             methods: METHODS.iter().map(|i| i.to_string()).collect(),
//         })),
//     }
// }
