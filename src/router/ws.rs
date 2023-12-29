use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use crate::state::AppState;

use super::handlers;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize)]
pub struct WsRequest {
    pub event: WsRequestEvent,
    pub body: Value,
}

#[derive(Debug, Deserialize)]
pub enum WsRequestEvent {
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
}

// impl WsRequestEvent {
//     pub fn from_str(s: &str) -> Option<Self> {
//         match s {
//             "admin-info" => Some(Self::AdminInfo),
//             "admin-backup" => Some(Self::AdminBackup),
//             "admin-config" => Some(Self::AdminConfig),
//             "admin-discover-version" => Some(Self::AdminDiscoverVersion),
//             "admin-module" => Some(Self::AdminModule),
//             "admin-restore" => Some(Self::AdminRestore),
//             "admin-list-operations" => Some(Self::AdminListOperations),
//             "mint-reissue" => Some(Self::MintReissue),
//             "mint-spend" => Some(Self::MintSpend),
//             "mint-validate" => Some(Self::MintValidate),
//             "mint-split" => Some(Self::MintSplit),
//             "mint-combine" => Some(Self::MintCombine),
//             "ln-invoice" => Some(Self::LnInvoice),
//             "ln-await-invoice" => Some(Self::LnAwaitInvoice),
//             "ln-pay" => Some(Self::LnPay),
//             "ln-await-pay" => Some(Self::LnAwaitPay),
//             "ln-list-gateways" => Some(Self::LnListGateways),
//             "ln-switch-gateway" => Some(Self::LnSwitchGateway),
//             "wallet-deposit-address" => Some(Self::WalletDepositAddress),
//             "wallet-await-deposit" => Some(Self::WalletAwaitDeposit),
//             "wallet-withdraw" => Some(Self::WalletWithdraw),
//             _ => None,
//         }
//     }
// }

/// All websocket request events are of the form:
/// {
///   "event": "event-name",
///   "body": { ... }
/// }
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            info!("Received: {}", text);
            let req = match serde_json::from_str::<WsRequest>(&text) {
                Ok(event) => event,
                Err(_) => {
                    socket
                        .send(Message::Text("Bad request".to_string()))
                        .await
                        .unwrap();
                    continue;
                }
            };
            let response = match req.event {
                WsRequestEvent::AdminInfo => {
                    handlers::fedimint::admin::info::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::AdminBackup => {
                    handlers::fedimint::admin::backup::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::AdminConfig => {
                    handlers::fedimint::admin::config::handle_ws(state.clone()).await
                }
                WsRequestEvent::AdminDiscoverVersion => {
                    handlers::fedimint::admin::discover_version::handle_ws(state.clone()).await
                }
                WsRequestEvent::AdminModule => {
                    handlers::fedimint::admin::module::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::AdminRestore => {
                    handlers::fedimint::admin::restore::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::AdminListOperations => {
                    handlers::fedimint::admin::list_operations::handle_ws(req.body, state.clone())
                        .await
                }
                WsRequestEvent::MintReissue => {
                    handlers::fedimint::mint::reissue::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::MintSpend => {
                    handlers::fedimint::mint::spend::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::MintValidate => {
                    handlers::fedimint::mint::validate::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::MintSplit => {
                    handlers::fedimint::mint::split::handle_ws(req.body).await
                }
                WsRequestEvent::MintCombine => {
                    handlers::fedimint::mint::combine::handle_ws(req.body).await
                }
                WsRequestEvent::LnInvoice => {
                    handlers::fedimint::ln::invoice::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::LnAwaitInvoice => {
                    handlers::fedimint::ln::await_invoice::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::LnPay => {
                    handlers::fedimint::ln::pay::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::LnAwaitPay => {
                    handlers::fedimint::ln::await_pay::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::LnListGateways => {
                    handlers::fedimint::ln::list_gateways::handle_ws(state.clone()).await
                }
                WsRequestEvent::LnSwitchGateway => {
                    handlers::fedimint::ln::switch_gateway::handle_ws(req.body, state.clone()).await
                }
                WsRequestEvent::WalletDepositAddress => {
                    handlers::fedimint::wallet::deposit_address::handle_ws(req.body, state.clone())
                        .await
                }
                WsRequestEvent::WalletAwaitDeposit => {
                    handlers::fedimint::wallet::await_deposit::handle_ws(req.body, state.clone())
                        .await
                }
                WsRequestEvent::WalletWithdraw => {
                    handlers::fedimint::wallet::withdraw::handle_ws(req.body, state.clone()).await
                }
            };
            match response {
                Ok(res) => {
                    socket.send(res).await.unwrap();
                }
                Err(e) => {
                    socket.send(Message::Text(e.to_string())).await.unwrap();
                }
            }
        }
    }
}
