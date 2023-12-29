use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::stream::StreamExt;
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

pub enum WsCommand {
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

impl WsCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "/admin/info" => Some(Self::AdminInfo),
            "/admin/backup" => Some(Self::AdminBackup),
            "/admin/config" => Some(Self::AdminConfig),
            "/admin/discover-version" => Some(Self::AdminDiscoverVersion),
            "/admin/module" => Some(Self::AdminModule),
            "/admin/restore" => Some(Self::AdminRestore),
            "/admin/list-operations" => Some(Self::AdminListOperations),
            "/mint/reissue" => Some(Self::MintReissue),
            "/mint/spend" => Some(Self::MintSpend),
            "/mint/validate" => Some(Self::MintValidate),
            "/mint/split" => Some(Self::MintSplit),
            "/mint/combine" => Some(Self::MintCombine),
            "/ln/invoice" => Some(Self::LnInvoice),
            "/ln/await-invoice" => Some(Self::LnAwaitInvoice),
            "/ln/pay" => Some(Self::LnPay),
            "/ln/await-pay" => Some(Self::LnAwaitPay),
            "/ln/list-gateways" => Some(Self::LnListGateways),
            "/ln/switch-gateway" => Some(Self::LnSwitchGateway),
            "/wallet/deposit-address" => Some(Self::WalletDepositAddress),
            "/wallet/await-deposit" => Some(Self::WalletAwaitDeposit),
            "/wallet/withdraw" => Some(Self::WalletWithdraw),
            _ => None,
        }
    }
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            info!("Received: {}", text);
            let v: Value = serde_json::from_str(&text).unwrap();
            let command = WsCommand::from_str(v["command"].as_str().unwrap());
            if let Some(command) = command {
                let response = match command {
                    WsCommand::AdminInfo => {
                        handlers::fedimint::admin::info::handle_ws(v["body"].clone(), state.clone())
                            .await
                    }
                    WsCommand::AdminBackup => {
                        handlers::fedimint::admin::backup::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::AdminConfig => {
                        handlers::fedimint::admin::config::handle_ws(state.clone()).await
                    }
                    WsCommand::AdminDiscoverVersion => {
                        handlers::fedimint::admin::discover_version::handle_ws(state.clone()).await
                    }
                    WsCommand::AdminModule => {
                        handlers::fedimint::admin::module::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::AdminRestore => {
                        handlers::fedimint::admin::restore::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::AdminListOperations => {
                        handlers::fedimint::admin::list_operations::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::MintReissue => {
                        handlers::fedimint::mint::reissue::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::MintSpend => {
                        handlers::fedimint::mint::spend::handle_ws(v["body"].clone(), state.clone())
                            .await
                    }
                    WsCommand::MintValidate => {
                        handlers::fedimint::mint::validate::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::MintSplit => {
                        handlers::fedimint::mint::split::handle_ws(v["body"].clone()).await
                    }
                    WsCommand::MintCombine => {
                        handlers::fedimint::mint::combine::handle_ws(v["body"].clone()).await
                    }
                    WsCommand::LnInvoice => {
                        handlers::fedimint::ln::invoice::handle_ws(v["body"].clone(), state.clone())
                            .await
                    }
                    WsCommand::LnAwaitInvoice => {
                        handlers::fedimint::ln::await_invoice::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::LnPay => {
                        handlers::fedimint::ln::pay::handle_ws(v["body"].clone(), state.clone())
                            .await
                    }
                    WsCommand::LnAwaitPay => {
                        handlers::fedimint::ln::await_pay::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::LnListGateways => {
                        handlers::fedimint::ln::list_gateways::handle_ws(state.clone()).await
                    }
                    WsCommand::LnSwitchGateway => {
                        handlers::fedimint::ln::switch_gateway::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::WalletDepositAddress => {
                        handlers::fedimint::wallet::deposit_address::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::WalletAwaitDeposit => {
                        handlers::fedimint::wallet::await_deposit::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
                    }
                    WsCommand::WalletWithdraw => {
                        handlers::fedimint::wallet::withdraw::handle_ws(
                            v["body"].clone(),
                            state.clone(),
                        )
                        .await
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
}
