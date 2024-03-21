use std::u64;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use anyhow::anyhow;
use std::string::String;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
enum MeltQuoteUnit {
    Sat,
}
#[derive(Debug, Deserialize)]
pub struct PostMeltQuoteRequest {
    request: String,
    unit: MeltQuoteUnit,
}
#[derive(Debug, Serialize)]
pub struct PostMeltQuoteResponse {
    quote: String,
    amount: u64,
    fee_reserve: u64,
    paid: bool,
    expiry: u64,
}

#[axum_macros::debug_handler]
pub async fn handle_method(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function

    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_method_quote_id(
    State(state): State<AppState>,
    Json(req): Json<PostMeltQuoteRequest>,
) -> Result<Json<PostMeltQuoteResponse>, AppError> {
    // TODO: Implement this function
    let client = match state.multimint.get_default().await {
        Some(client) => Ok(client),
            None => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                anyhow!("No default client found "),)),
    };

    let amount=client.as_ref().unwrap().get_balance().await;

    let response= PostMeltQuoteResponse{
        quote: "me".to_string(),
        amount: amount.msats,
        fee_reserve: 2,
        paid: true,
        expiry: 1701704757,

    };

    Ok(Json(response))
}
