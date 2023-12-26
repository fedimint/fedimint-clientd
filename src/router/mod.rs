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
/// - `/fedimint/v2/admin/info`: Display wallet info (holdings, tiers).
/// - `/fedimint/v2/admin/backup`: Upload the (encrypted) snapshot of mint notes to federation.
/// - `/fedimint/v2/admin/discover-version`: Discover the common api version to use to communicate with the federation.
/// - `/fedimint/v2/admin/restore`: Restore the previously created backup of mint notes (with `backup` command).
/// - `/fedimint/v2/admin/list-operations`: List operations.
/// - `/fedimint/v2/admin/module`: Call a module subcommand.
/// - `/fedimint/v2/admin/config`: Returns the client config.
///
/// Mint related commands:
/// - `/fedimint/v2/mint/reissue`: Reissue notes received from a third party to avoid double spends.
/// - `/fedimint/v2/mint/spend`: Prepare notes to send to a third party as a payment.
/// - `/fedimint/v2/mint/validate`: Verifies the signatures of e-cash notes, but *not* if they have been spent already.
/// - `/fedimint/v2/mint/split`: Splits a string containing multiple e-cash notes (e.g. from the `spend` command) into ones that contain exactly one.
/// - `/fedimint/v2/mint/combine`: Combines two or more serialized e-cash notes strings.
///
/// Lightning network related commands:
/// - `/fedimint/v2/ln/invoice`: Create a lightning invoice to receive payment via gateway.
/// - `/fedimint/v2/ln/await-invoice`: Wait for incoming invoice to be paid.
/// - `/fedimint/v2/ln/pay`: Pay a lightning invoice or lnurl via a gateway.
/// - `/fedimint/v2/ln/await-pay`: Wait for a lightning payment to complete.
/// - `/fedimint/v2/ln/list-gateways`: List registered gateways.
/// - `/fedimint/v2/ln/switch-gateway`: Switch active gateway.
///
/// Onchain related commands:
/// - `/fedimint/v2/onchain/deposit-address`: Generate a new deposit address, funds sent to it can later be claimed.
/// - `/fedimint/v2/onchain/await-deposit`: Wait for deposit on previously generated address.
/// - `/fedimint/v2/onchain/withdraw`: Withdraw funds from the federation.
fn fedimint_v2_router() -> Router<AppState> {
    let mint_router = Router::new()
        .route("/reissue", post(fedimint::mint::reissue::handle_reissue))
        .route("/spend", post(fedimint::mint::spend::handle_spend))
        .route("/validate", post(fedimint::mint::validate::handle_validate))
        .route("/split", post(fedimint::mint::split::handle_split))
        .route("/combine", post(fedimint::mint::combine::handle_combine));

    let ln_router = Router::new()
        .route("/invoice", post(fedimint::ln::invoice::handle_invoice))
        .route(
            "/await-invoice",
            post(fedimint::ln::await_invoice::handle_await_invoice),
        )
        .route("/pay", post(fedimint::ln::pay::handle_pay))
        .route(
            "/await-pay",
            post(fedimint::ln::await_pay::handle_await_pay),
        )
        .route(
            "/list-gateways",
            get(fedimint::ln::list_gateways::handle_list_gateways),
        )
        .route(
            "/switch-gateway",
            post(fedimint::ln::switch_gateway::handle_switch_gateway),
        );

    let onchain_router = Router::new()
        .route(
            "/deposit-address",
            post(fedimint::onchain::deposit_address::handle_deposit_address),
        )
        .route(
            "/await-deposit",
            post(fedimint::onchain::await_deposit::handle_await_deposit),
        )
        .route(
            "/withdraw",
            post(fedimint::onchain::withdraw::handle_withdraw),
        );

    let admin_router = Router::new()
        .route("/info", get(fedimint::admin::info::handle_info))
        .route("/backup", post(fedimint::admin::backup::handle_backup))
        .route(
            "/discover-version",
            get(fedimint::admin::discover_version::handle_discover_version),
        )
        .route("/restore", post(fedimint::admin::restore::handle_restore))
        // .route("/printsecret", get(fedimint::handle_printsecret)) TODO: should I expose this under admin?
        .route(
            "/list-operations",
            get(fedimint::admin::list_operations::handle_list_operations),
        )
        .route("/module", post(fedimint::admin::module::handle_module))
        .route("/config", get(fedimint::admin::config::handle_config));

    let base_router = Router::new()
        .nest("/admin", admin_router)
        .nest("/mint", mint_router)
        .nest("/ln", ln_router)
        .nest("/onchain", onchain_router);

    base_router
}

/// Implements Cashu V1 API Routes:
///
/// REQUIRED
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
///
/// OPTIONAL
/// NUT-07 Token State Check
/// - `/cashu/v1/check`
/// NUT-08 Lightning Fee Return
/// - Modification of NUT-05 Melt
/// NUT-10 Spending Conditions
/// NUT-11 Pay to Public Key (P2PK)
/// - Fedimint already does this
/// NUT-12 Offline Ecash Signature Validation
/// - DLEQ in BlindedSignature for Mint to User
fn cashu_v1_router() -> Router<AppState> {
    let cashu_router = Router::new()
        .route("/keys", get(cashu::keys::handle_keys))
        .route("/keys/{keyset_id}", get(cashu::keys::handle_keys_keyset_id))
        .route("/keysets", get(cashu::keysets::handle_keysets))
        .route("/swap", post(cashu::swap::handle_swap))
        .route("/mint/quote/{method}", get(cashu::mint::handle_mint_quote))
        .route(
            "/mint/quote/{method}/{quote_id}",
            get(cashu::mint::handle_mint_quote_quote_id),
        )
        .route("/mint/{method}", post(cashu::mint::handle_mint))
        .route("/melt/quote/{method}", get(cashu::melt::handle_melt_quote))
        .route(
            "/melt/quote/{method}/{quote_id}",
            get(cashu::melt::handle_melt_quote_quote_id),
        )
        .route("/info", get(cashu::info::handle_info))
        .route("/check", post(cashu::check::handle_check));
    cashu_router
}
