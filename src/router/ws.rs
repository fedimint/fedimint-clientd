use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::state::AppState;

use super::handlers;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WsMessage {
    pub event: WsEvent,
    pub msg: Value,
    pub code: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WsEvent {
    AdminInfo,
    AdminBackup,
    AdminConfig,
    AdminDiscoverVersion,
    AdminModule,
    AdminRestore,
    AdminListOperations,
    MintReissue,
    MintSpend,
    MintValidate,
    MintSplit,
    MintCombine,
    LnInvoice,
    LnAwaitInvoice,
    LnPay,
    LnAwaitPay,
    LnListGateways,
    LnSwitchGateway,
    WalletDepositAddress,
    WalletAwaitDeposit,
    WalletWithdraw,
    Error,
}

/// All websocket request events are of the form:
/// {
///   "event": "event-name", // kebab case e.g. "admin-info" or "mint-validate"
///   "msg": { ... }
/// }
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            info!("Received: {}", text);
            let req = match serde_json::from_str::<WsMessage>(&text) {
                Ok(event) => event,
                Err(err) => {
                    let err_msg = WsMessage {
                        event: WsEvent::Error,
                        msg: json!({ "error": err.to_string() }),
                        code: Some(400),
                    };
                    socket
                        .send(Message::Text(serde_json::to_string(&err_msg).unwrap()))
                        .await
                        .unwrap();
                    continue;
                }
            };
            let response_msg = match req.event {
                WsEvent::AdminInfo => {
                    handlers::fedimint::admin::info::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::AdminBackup => {
                    handlers::fedimint::admin::backup::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::AdminConfig => {
                    handlers::fedimint::admin::config::handle_ws(state.clone()).await
                }
                WsEvent::AdminDiscoverVersion => {
                    handlers::fedimint::admin::discover_version::handle_ws(state.clone()).await
                }
                WsEvent::AdminModule => {
                    handlers::fedimint::admin::module::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::AdminRestore => {
                    handlers::fedimint::admin::restore::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::AdminListOperations => {
                    handlers::fedimint::admin::list_operations::handle_ws(req.msg, state.clone())
                        .await
                }
                WsEvent::MintReissue => {
                    handlers::fedimint::mint::reissue::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::MintSpend => {
                    handlers::fedimint::mint::spend::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::MintValidate => {
                    handlers::fedimint::mint::validate::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::MintSplit => handlers::fedimint::mint::split::handle_ws(req.msg).await,
                WsEvent::MintCombine => handlers::fedimint::mint::combine::handle_ws(req.msg).await,
                WsEvent::LnInvoice => {
                    handlers::fedimint::ln::invoice::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::LnAwaitInvoice => {
                    handlers::fedimint::ln::await_invoice::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::LnPay => {
                    handlers::fedimint::ln::pay::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::LnAwaitPay => {
                    handlers::fedimint::ln::await_pay::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::LnListGateways => {
                    handlers::fedimint::ln::list_gateways::handle_ws(state.clone()).await
                }
                WsEvent::LnSwitchGateway => {
                    handlers::fedimint::ln::switch_gateway::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::WalletDepositAddress => {
                    handlers::fedimint::wallet::deposit_address::handle_ws(req.msg, state.clone())
                        .await
                }
                WsEvent::WalletAwaitDeposit => {
                    handlers::fedimint::wallet::await_deposit::handle_ws(req.msg, state.clone())
                        .await
                }
                WsEvent::WalletWithdraw => {
                    handlers::fedimint::wallet::withdraw::handle_ws(req.msg, state.clone()).await
                }
                WsEvent::Error => {
                    let err_msg = WsMessage {
                        event: WsEvent::Error,
                        msg: json!({ "error": "Unknown event" }),
                        code: Some(400),
                    };
                    socket
                        .send(Message::Text(serde_json::to_string(&err_msg).unwrap()))
                        .await
                        .unwrap();
                    continue;
                }
            };
            match response_msg {
                Ok(res) => {
                    let res_msg = WsMessage {
                        event: req.event,
                        msg: res,
                        code: Some(200),
                    };

                    socket
                        .send(Message::Text(serde_json::to_string(&res_msg).unwrap()))
                        .await
                        .unwrap();
                }
                Err(e) => {
                    socket.send(Message::Text(e.to_string())).await.unwrap();
                }
            }
        }
    }
}
