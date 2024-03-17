use anyhow::anyhow;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub enum MeltUnit {
    #[serde(rename = "sat")]
    Sat,
}

#[derive(Debug, Deserialize)]
pub struct PostMeltQuoteRequest {
    pub request: String,
    pub unit: MeltUnit,
}

#[derive(Debug, Serialize)]
pub struct PostMeltQuoteResponse {
    quote: String,
    amount: u64,
    fee_reserve: u64,
    paid: bool,
    expiry: i64,
}

#[axum_macros::debug_handler]
pub async fn handle_method(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<PostMeltQuoteRequest>,
) -> Result<Json<PostMeltQuoteResponse>, AppError> {
    info!("Received request: {:?}", req);
    let _client = match state.multimint.get_by_str("test").await {
        Some(client) => client,
        None => {
            return Err(AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("No default client found"),
            ))
        }
    };

    let response = PostMeltQuoteResponse {
        quote: "quote".to_string(),
        amount: 0,
        fee_reserve: 0,
        paid: false,
        expiry: 0,
    };

    Ok(Json(response))
}
