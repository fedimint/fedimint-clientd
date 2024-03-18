use std::convert::Infallible;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::extract::ws::{WebSocket, Message as WSMessage};
use fedimint_client::ClientArc;
use fedimint_core::config::FederationId;
use fedimint_core::core::OperationId;
use fedimint_wallet_client::{DepositState, WalletClientModule};
use futures_util:: StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitDepositRequest {
    pub operation_id: OperationId,
    pub federation_id: Option<FederationId>,
}

#[derive(Debug, Serialize)] 
#[serde(rename_all = "camelCase")]
pub struct AwaitDepositResponse {
    pub status: DepositState,
}

pub async fn handle_ws(
    mut ws: WebSocket,
    state: AppState,
) -> Result<(), Infallible> {
    while let Some(message) = ws.next().await {
        match message {
            Ok(WSMessage::Text(text)) => {
                let req: Result<AwaitDepositRequest, _> = serde_json::from_str(&text);
                
                let client_result = if let Ok(req) = req.as_ref() {
                    match state.get_client(req.federation_id).await {
                        Ok(client) => Some(client),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                if let (Ok(req), Some(client)) = (req, client_result) {
                    match handle_deposit_request(client, req.operation_id).await {
                        Ok(deposit_state) => {
                            let response = serde_json::to_string(&AwaitDepositResponse { status: deposit_state })
                                .unwrap_or_else(|_| json!({ "error": "Serialization error" }).to_string());
                            ws.send(WSMessage::Text(response)).await.ok(); 
                        },
                        Err(_) => {
                            let error_message = json!({ "error": "Error processing deposit request" }).to_string();
                            ws.send(WSMessage::Text(error_message)).await.ok(); 
                        }
                    }
                } else {
                    let error_message = json!({ "error": "Invalid request or client retrieval failed" }).to_string();
                    ws.send(WSMessage::Text(error_message)).await.ok(); 
                }
            },
            _ => continue, 
        }   
    }

    Ok(())
}

async fn handle_deposit_request(client: ClientArc, operation_id: OperationId) -> Result<DepositState, AppError> {
    let updates = client
        .get_first_module::<WalletClientModule>()
        .subscribe_deposit_updates(operation_id)
        .await
        .map_err(|_| AppError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_stream();

    updates.fold(None, |_, update| async move { Some(update) }).await
        .ok_or_else(|| AppError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))
}

pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitDepositRequest>,
) -> Result<Json<AwaitDepositResponse>, AppError> {
    let client = state.get_client(req.federation_id).await?;
    let deposit_state = handle_deposit_request(client, req.operation_id).await?;
    
    Ok(Json(AwaitDepositResponse { status: deposit_state }))
}

