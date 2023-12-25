use anyhow::Result;
use axum::routing::post;
use axum::{http::Method, routing::get, Router};
pub mod handlers;

use handlers::*;
use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::{config::CONFIG, state::AppState};

/// Create a router with various routes.
///
/// The routes include:
/// - `/`: Display the README.
/// - `/api/info`: Display wallet info (holdings, tiers).
/// - `/api/reissue`: Reissue notes received from a third party to avoid double spends.
/// - `/api/spend`: Prepare notes to send to a third party as a payment.
/// - `/api/validate`: Verifies the signatures of e-cash notes, but *not* if they have been spent already.
/// - `/api/split`: Splits a string containing multiple e-cash notes (e.g. from the `spend` command) into ones that contain exactly one.
/// - `/api/combine`: Combines two or more serialized e-cash notes strings.
/// - `/api/lninvoice`: Create a lightning invoice to receive payment via gateway.
/// - `/api/awaitinvoice`: Wait for incoming invoice to be paid.
/// - `/api/lnpay`: Pay a lightning invoice or lnurl via a gateway.
/// - `/api/awaitlnpay`: Wait for a lightning payment to complete.
/// - `/api/listgateways`: List registered gateways.
/// - `/api/switchgateway`: Switch active gateway.
/// - `/api/depositaddress`: Generate a new deposit address, funds sent to it can later be claimed.
/// - `/api/awaitdeposit`: Wait for deposit on previously generated address.
/// - `/api/withdraw`: Withdraw funds from the federation.
/// - `/api/backup`: Upload the (encrypted) snapshot of mint notes to federation.
/// - `/api/discoverversion`: Discover the common api version to use to communicate with the federation.
/// - `/api/restore`: Restore the previously created backup of mint notes (with `backup` command).
/// - `/api/printsecret`: Print the secret key of the client.
/// - `/api/listoperations`: List operations.
/// - `/api/module`: Call a module subcommand.
/// - `/api/config`: Returns the client config.
pub async fn create_router(state: AppState) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(handle_readme))
        .route("/api/info", get(fedimint::handle_info))
        .route("/api/reissue", post(fedimint::handle_reissue))
        .route("/api/spend", post(fedimint::handle_spend))
        .route("/api/validate", post(fedimint::handle_validate))
        .route("/api/split", post(fedimint::handle_split))
        .route("/api/combine", post(fedimint::handle_combine))
        .route("/api/lninvoice", post(fedimint::handle_lninvoice))
        .route("/api/awaitinvoice", post(fedimint::handle_awaitinvoice))
        .route("/api/lnpay", post(fedimint::handle_lnpay))
        .route("/api/awaitlnpay", post(fedimint::handle_awaitlnpay))
        .route("/api/listgateways", get(fedimint::handle_listgateways))
        .route("/api/switchgateway", post(fedimint::handle_switchgateway))
        .route("/api/depositaddress", post(fedimint::handle_depositaddress))
        .route("/api/awaitdeposit", post(fedimint::handle_awaitdeposit))
        .route("/api/withdraw", post(fedimint::handle_withdraw))
        .route("/api/backup", post(fedimint::handle_backup))
        .route(
            "/api/discoverversion",
            get(fedimint::handle_discoverversion),
        )
        .route("/api/restore", post(fedimint::handle_restore))
        .route("/api/printsecret", get(fedimint::handle_printsecret))
        .route("/api/listoperations", get(fedimint::handle_listoperations))
        .route("/api/module", post(fedimint::handle_module))
        .route("/api/config", get(fedimint::handle_config))
        .with_state(state)
        .layer(cors)
        .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password));

    Ok(app)
}
