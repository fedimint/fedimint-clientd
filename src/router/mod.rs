use anyhow::Result;
use axum::routing::post;
use axum::{routing::get, Router};
pub mod handlers;

use handlers::*;
// use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::{config::CONFIG, state::AppState};

pub async fn create_router(state: AppState) -> Result<Router> {
    // TODO: Allow CORS? Probably not, since this should just interact with the local machine.
    // let cors = CorsLayer::new()
    //     .allow_methods([Method::GET, Method::POST])
    //     .allow_origin(Any);

    let app = Router::new()
        .route("/", get(handle_readme))
        .nest("/fedimint/v2", fedimint_v2_router())
        .nest("/cashu/v1", cashu_v1_router())
        .with_state(state)
        // .layer(cors)
        .layer(ValidateRequestHeaderLayer::bearer(&CONFIG.password));

    Ok(app)
}

/// Implements Fedimint V0.2 API Route matching against CLI commands:
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
fn fedimint_v2_router() -> Router<AppState> {
    Router::new()
        .route("/info", get(fedimint::handle_info))
        .route("/reissue", post(fedimint::handle_reissue))
        .route("/spend", post(fedimint::handle_spend))
        .route("/validate", post(fedimint::handle_validate))
        .route("/split", post(fedimint::handle_split))
        .route("/combine", post(fedimint::handle_combine))
        .route("/lninvoice", post(fedimint::handle_lninvoice))
        .route("/awaitinvoice", post(fedimint::handle_awaitinvoice))
        .route("/lnpay", post(fedimint::handle_lnpay))
        .route("/awaitlnpay", post(fedimint::handle_awaitlnpay))
        .route("/listgateways", get(fedimint::handle_listgateways))
        .route("/switchgateway", post(fedimint::handle_switchgateway))
        .route("/depositaddress", post(fedimint::handle_depositaddress))
        .route("/awaitdeposit", post(fedimint::handle_awaitdeposit))
        .route("/withdraw", post(fedimint::handle_withdraw))
        .route("/backup", post(fedimint::handle_backup))
        .route("/discoverversion", get(fedimint::handle_discoverversion))
        .route("/restore", post(fedimint::handle_restore))
        // .route("/printsecret", get(fedimint::handle_printsecret))
        .route("/listoperations", get(fedimint::handle_listoperations))
        .route("/module", post(fedimint::handle_module))
        .route("/config", get(fedimint::handle_config))
}

/// Implements Cashu V1 API Routes:
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
/// NUT-07 Token State Check
/// - `/cashu/v1/check`
///
fn cashu_v1_router() -> Router<AppState> {
    let cashu_router = Router::new()
        .route("/keys", get(cashu::handle_keys))
        .route("/keys/{keyset_id}", get(cashu::handle_keys_keyset_id))
        .route("/keysets", get(cashu::handle_keysets))
        .route("/swap", post(cashu::handle_swap))
        .route("/mint/quote/{method}", get(cashu::handle_mint_quote))
        .route(
            "/mint/quote/{method}/{quote_id}",
            get(cashu::handle_mint_quote_quote_id),
        )
        .route("/mint/{method}", post(cashu::handle_mint))
        .route("/melt/quote/{method}", get(cashu::handle_melt_quote))
        .route(
            "/melt/quote/{method}/{quote_id}",
            get(cashu::handle_melt_quote_quote_id),
        )
        .route("/info", get(cashu::handle_info))
        .route("/check", post(cashu::handle_check));
    cashu_router
}
