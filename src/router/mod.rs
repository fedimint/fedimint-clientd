use anyhow::Result;
use axum::routing::post;
use axum::{http::Method, routing::get, Router};
pub mod handlers;

use handlers::*;
use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::{config::CONFIG, state::AppState};

/// - `/`: Display the README.
///
/// Implements Fedimint API Route matching against CLI commands:
/// - `/fedimint/api/info`: Display wallet info (holdings, tiers).
/// - `/fedimint/api/reissue`: Reissue notes received from a third party to avoid double spends.
/// - `/fedimint/api/spend`: Prepare notes to send to a third party as a payment.
/// - `/fedimint/api/validate`: Verifies the signatures of e-cash notes, but *not* if they have been spent already.
/// - `/fedimint/api/split`: Splits a string containing multiple e-cash notes (e.g. from the `spend` command) into ones that contain exactly one.
/// - `/fedimint/api/combine`: Combines two or more serialized e-cash notes strings.
/// - `/fedimint/api/lninvoice`: Create a lightning invoice to receive payment via gateway.
/// - `/fedimint/api/awaitinvoice`: Wait for incoming invoice to be paid.
/// - `/fedimint/api/lnpay`: Pay a lightning invoice or lnurl via a gateway.
/// - `/fedimint/api/awaitlnpay`: Wait for a lightning payment to complete.
/// - `/fedimint/api/listgateways`: List registered gateways.
/// - `/fedimint/api/switchgateway`: Switch active gateway.
/// - `/fedimint/api/depositaddress`: Generate a new deposit address, funds sent to it can later be claimed.
/// - `/fedimint/api/awaitdeposit`: Wait for deposit on previously generated address.
/// - `/fedimint/api/withdraw`: Withdraw funds from the federation.
/// - `/fedimint/api/backup`: Upload the (encrypted) snapshot of mint notes to federation.
/// - `/fedimint/api/discoverversion`: Discover the common api version to use to communicate with the federation.
/// - `/fedimint/api/restore`: Restore the previously created backup of mint notes (with `backup` command).
/// - `/fedimint/api/printsecret`: Print the secret key of the client.
/// - `/fedimint/api/listoperations`: List operations.
/// - `/fedimint/api/module`: Call a module subcommand.
/// - `/fedimint/api/config`: Returns the client config.
///
/// Implements Cashu API Routes:
/// NUT-01 Mint Public Key Exchange && NUT-02 Keysets and Keyset IDs
/// - `/cashu/v1/keys`
/// - `/cashu/v1/keys/{keyset_id}`
/// - `/cashu/v1/keysets`
/// NUT-03 Swap Tokens (Equivalent to `reissue` command)
/// - `/cashu/v1/swap`
/// NUT-04 Mint Tokens: supports `bolt11` and `onchain` methods
/// - `/cashu/v1/mint/quote/{method}`
/// - `/cashu/v1/mint/quote/{method}/{quote_id}`
/// - `/cashu/v1/mint/{method}`
/// NUT-05 Melting Tokens: supports `bolt11` and `onchain` methods
/// - `/cashu/v1/melt/quote/{method}`
/// - `/cashu/v1/melt/quote/{method}/{quote_id}`
/// NUT-06 Mint Information
/// - `/cashu/v1/info`
pub async fn create_router(state: AppState) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let fedimint_router = Router::new()
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
        // .route("/api/printsecret", get(fedimint::handle_printsecret))
        .route("/api/listoperations", get(fedimint::handle_listoperations))
        .route("/api/module", post(fedimint::handle_module))
        .route("/api/config", get(fedimint::handle_config));

    let cashu_router = Router::new()
        .route("/v1/keys", get(cashu::handle_keys))
        .route("/v1/keys/{keyset_id}", get(cashu::handle_keys_keyset_id))
        .route("/v1/keysets", get(cashu::handle_keysets))
        .route("/v1/swap", post(cashu::handle_swap))
        .route("/v1/mint/quote/{method}", get(cashu::handle_mint_quote))
        .route(
            "/v1/mint/quote/{method}/{quote_id}",
            get(cashu::handle_mint_quote_quote_id),
        )
        .route("/v1/mint/{method}", post(cashu::handle_mint))
        .route("/v1/melt/quote/{method}", get(cashu::handle_melt_quote))
        .route(
            "/v1/melt/quote/{method}/{quote_id}",
            get(cashu::handle_melt_quote_quote_id),
        )
        .route("/v1/info", get(cashu::handle_info));

    let app = Router::new()
        .nest("/fedimint", fedimint_router)
        .nest("/cashu", cashu_router)
        .with_state(state)
        .layer(cors)
        .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password));

    Ok(app)
}
