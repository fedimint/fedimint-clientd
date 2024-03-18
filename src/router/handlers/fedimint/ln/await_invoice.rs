use anyhow::anyhow; // For detailed error messages
use axum::extract::ws::{WebSocket, Message as WSMessage};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use fedimint_client::ClientArc;
use fedimint_core::config::FederationId;
use fedimint_core::core::OperationId;
use fedimint_ln_client::{LightningClientModule, LnReceiveState}; // Assuming this is the correct module
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible; // If there's no scenario where this async fn can fail

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceRequest {
    pub operation_id: OperationId,
    pub federation_id: Option<FederationId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AwaitInvoiceResponse {
    pub status: LnReceiveState,
}

pub async fn handle_ws(
    mut ws: WebSocket,
    state: AppState,
) -> Result<(), Infallible> {
    while let Some(message) = ws.next().await {
        match message {
            Ok(WSMessage::Text(text)) => {
                let req: Result<AwaitInvoiceRequest, _> = serde_json::from_str(&text);

                let client_result = if let Ok(req) = req.as_ref() {
                    match state.get_client(req.federation_id).await {
                        Ok(client) => Some(client),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                if let (Ok(req), Some(client)) = (req, client_result) {
                    match handle_invoice_request(client, req.operation_id).await {
                        Ok(invoice_state) => {
                            let response = serde_json::to_string(&AwaitInvoiceResponse { status: invoice_state })
                                .unwrap_or_else(|_| json!({ "error": "Serialization error" }).to_string());
                            ws.send(WSMessage::Text(response)).await.ok(); 
                        },
                        Err(_) => {
                            let error_message = json!({ "error": "Error processing invoice request" }).to_string();
                            ws.send(WSMessage::Text(error_message)).await.ok(); 
                        }
                    }
                } else {
                    let error_message = json!({ "error": "Invalid request or client retrieval failed" }).to_string();
                    ws.send(WSMessage::Text(error_message)).await.ok(); 
                }
            },
            _ => continue, // Non-text messages are ignored
        }
    }

    Ok(())
}

async fn handle_invoice_request(client: ClientArc, operation_id: OperationId) -> Result<LnReceiveState, AppError> {

    let updates = client
        .get_first_module::<LightningClientModule>()
        .subscribe_ln_receive(operation_id)
        .await
        .map_err(|_| AppError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?
        .into_stream();

    updates.fold(None, |_, update| async move { Some(update) }).await
        .ok_or_else(|| AppError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))
}

pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<AwaitInvoiceRequest>,
) -> Result<Json<AwaitInvoiceResponse>, AppError> {
    // Retrieve the client using the federation ID provided in the request
    let client = state.get_client(req.federation_id).await
        .map_err(|_| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("Failed to get client")))?;

    // Handle the invoice request and await its completion
    let invoice_state = handle_invoice_request(client, req.operation_id).await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("Error handling invoice request: {}", e)))?;
    
    // Return the invoice response as JSON
    Ok(Json(AwaitInvoiceResponse { status: invoice_state }))
}