use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
enum MintQuoteUnit {
    Sat,
}

#[derive(Debug, Deserialize)]
pub struct PostMintQuoteRequest {
    amount: u64,
    unit: MintQuoteUnit,

}

#[derive(Debug, Serialize)]
pub struct PostMintQuoteResponse {
    quote: String,
    request: String,
    paid: bool,
    expiry: u64,
}


#[axum_macros::debug_handler]
pub async fn handle_method(State(_state): State<AppState>) -> Result<(), AppError> {
    // TODO: Implement this function
    Ok(())
}

#[axum_macros::debug_handler]
pub async fn handle_method_quote_id(State(state): State<AppState>,
Json(req): Json<PostMintQuoteRequest>,
) -> Result<Json<PostMintQuoteResponse>, AppError> {
    // TODO: Implement this function

    let client = match state.multimint.get_default().await {
        Some(client) => Ok(client),
            None => Err(AppError::new(StatusCode::BAD_REQUEST, anyhow!("No default client"),)),
    };

    let response = PostMintQuoteResponse{
        quote: "hi".to_string(),
        request: "jhfdjm".to_string(),
        paid: false,
        expiry: 1012330,
    };

    Ok(Json(response))
}
