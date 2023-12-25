use anyhow::Result;
use axum::routing::post;
use axum::{http::Method, routing::get, Router};
pub mod handlers;

use handlers::*;
use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::{config::CONFIG, state::AppState};

pub async fn create_router(state: AppState) -> Result<Router> {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(handle_readme))
        .route("/api/federation_id", get(fedimint::handle_federation_id))
        .route("/api/info", get(fedimint::handle_info))
        .route("/api/reissue", post(fedimint::handle_reissue))
        .route("/api/spend", post(fedimint::handle_spend))
        .route("/api/validate", post(fedimint::handle_validate))
        .with_state(state)
        .layer(cors)
        .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password));

    Ok(app)
}
