use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;


use super::handlers;
use crate::error::AppError;
use crate::state::AppState;

const JSONRPC_VERSION: &str = "2.0";
const JSONRPC_ERROR_INVALID_REQUEST: i16 = -32600;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: JsonRpcMethod,
    pub params: Value,
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i16,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum JsonRpcMethod {
    AdminBackup,
    AdminConfig,
    AdminDiscoverVersion,
    AdminFederationIds,
    AdminInfo,
    AdminJoin,
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

async fn handle_socket(mut socket: WebSocket, state: AppState) {

    while let Some(message) = socket.next().await {
        if let Ok(Message::Text(text)) = message {
            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&text) {
                match req.method {
                    JsonRpcMethod::WalletAwaitDeposit => {
                        handlers::fedimint::wallet::await_deposit::handle_ws(socket, state.clone()).await;
                        break; 
                    },
                    JsonRpcMethod::LnAwaitInvoice => {
                        handlers::fedimint::ln::await_invoice::handle_ws(socket, state.clone()).await;
                        break; 
                    },
                    _ => {
                        // For other methods, use the match_method function to handle them accordingly.
                        let response = match_method(req.clone(), state.clone()).await;
                        let res_msg = create_json_rpc_response(response, req.id);
                        if let Err(send_error) = socket.send(res_msg).await {
                            eprintln!("Failed to send WebSocket message for method");
                        }
                    }
                }
            } else {
                // If the request couldn't be parsed, send an invalid request format error.
                send_err_invalid_req(&mut socket, "Invalid request format", &text).await;
            }
        }
    }
}

fn create_json_rpc_response(res: Result<Value, AppError>, req_id: u64) -> Message {
    let json_rpc_msg = match res {
        Ok(res) => JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(res),
            error: None,
            id: req_id,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: e.status.as_u16() as i16,
                message: e.error.to_string(),
            }),
            id: req_id,
        },
    };

    // TODO: Proper error handling for serialization, but this should never fail
    let msg_text = serde_json::to_string(&json_rpc_msg).map_err(|err| {
        "Internal Error - Failed to serialize JSON-RPC response: ".to_string() + &err.to_string()
    });

    Message::Text(msg_text.unwrap())
}

async fn send_err_invalid_req(socket: &mut WebSocket, err_message: &str, text: &str) {
    let id = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("id").cloned())
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let err_msg = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": JSONRPC_ERROR_INVALID_REQUEST,
            "message": err_message,
        },
        "id": id,
    }).to_string();

    if let Err(send_error) = socket.send(Message::Text(err_msg)).await {
        eprintln!("Failed to send error response for invalid request format {:?}", send_error);
    }
}

async fn match_method(req: JsonRpcRequest, state: AppState) -> Result<Value, AppError> {
    match req.method {
        JsonRpcMethod::AdminBackup => {
            handlers::fedimint::admin::backup::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminConfig => {
            handlers::fedimint::admin::config::handle_ws(state.clone()).await
        }
        JsonRpcMethod::AdminDiscoverVersion => {
            handlers::fedimint::admin::discover_version::handle_ws(state.clone()).await
        }
        JsonRpcMethod::AdminFederationIds => {
            handlers::fedimint::admin::federation_ids::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminInfo => {
            handlers::fedimint::admin::info::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminJoin => {
            handlers::fedimint::admin::join::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminModule => {
            handlers::fedimint::admin::module::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminRestore => {
            handlers::fedimint::admin::restore::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::AdminListOperations => {
            handlers::fedimint::admin::list_operations::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::MintReissue => {
            handlers::fedimint::mint::reissue::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::MintSpend => {
            handlers::fedimint::mint::spend::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::MintValidate => {
            handlers::fedimint::mint::validate::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::MintSplit => handlers::fedimint::mint::split::handle_ws(req.params).await,
        JsonRpcMethod::MintCombine => {
            handlers::fedimint::mint::combine::handle_ws(req.params).await
        }
        JsonRpcMethod::LnInvoice => {
            handlers::fedimint::ln::invoice::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::LnAwaitInvoice => {
            Ok(serde_json::Value::Null)
        }
        JsonRpcMethod::LnPay => {
            handlers::fedimint::ln::pay::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::LnAwaitPay => {
            handlers::fedimint::ln::await_pay::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::LnListGateways => {
            handlers::fedimint::ln::list_gateways::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::LnSwitchGateway => {
            handlers::fedimint::ln::switch_gateway::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::WalletDepositAddress => {
            handlers::fedimint::wallet::deposit_address::handle_ws(state.clone(), req.params).await
        }
        JsonRpcMethod::WalletAwaitDeposit => {
            Ok(serde_json::Value::Null)
        }
        JsonRpcMethod::WalletWithdraw => {
            handlers::fedimint::wallet::withdraw::handle_ws(state.clone(), req.params).await
        }
    }
}
