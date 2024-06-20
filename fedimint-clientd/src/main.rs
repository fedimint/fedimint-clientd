use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use axum::http::Method;
use multimint::fedimint_core::api::InviteCode;
use router::handlers::{admin, ln, mint, onchain};
use router::ws::websocket_handler;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod error;
mod router;
mod state;
mod utils;

use axum::routing::{get, post};
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use clap::{Parser, Subcommand, ValueEnum};
use state::AppState;
// use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Mode {
    Rest,
    Ws,
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rest" => Ok(Mode::Rest),
            "ws" => Ok(Mode::Ws),
            _ => Err(anyhow::anyhow!("Invalid mode")),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Kody Low")]
struct Cli {
    /// Federation invite code
    #[clap(long, env = "FEDIMINT_CLIENTD_INVITE_CODE", required = false)]
    invite_code: String,

    /// Path to FM database
    #[clap(long, env = "FEDIMINT_CLIENTD_WORK_DIR", required = true)]
    work_dir: PathBuf,

    /// Password
    #[clap(long, env = "FEDIMINT_CLIENTD_PASSWORD", required = true)]
    password: String,

    /// Addr
    #[clap(long, env = "FEDIMINT_CLIENTD_ADDR", required = true)]
    addr: String,

    /// Secret Key
    #[clap(long, env = "FEDIMINT_CLIENTD_SECRET_KEY", required = false)]
    secret_key: String,

    /// Mode: ws, rest
    #[clap(long, env = "FEDIMINT_CLIENTD_MODE", default_value = "rest")]
    mode: Mode,
}

// const PID_FILE: &str = "/tmp/fedimint_http.pid";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli: Cli = Cli::parse();

    let secret_key = hex::decode(cli.secret_key).map_err(|_| anyhow!("Invalid secret key"))?;
    let secret_key: [u8; 64] = secret_key
        .try_into()
        .map_err(|_| anyhow!("Invalid secret key"))?;

    let mut state = AppState::new(cli.work_dir, secret_key).await?;

    match InviteCode::from_str(&cli.invite_code) {
        Ok(invite_code) => {
            let federation_id = state.multimint.add_fedimint_client(invite_code).await?;
            info!("Created client for federation id: {:?}", federation_id);
        }
        Err(e) => {
            info!(
                "No federation invite code provided, skipping client creation: {}",
                e
            );
        }
    }

    if state.multimint.all().await.is_empty() {
        return Err(anyhow::anyhow!("No clients found, must have at least one client to start the server. Try providing a federation invite code with the `--invite-code` flag or setting the `FEDIMINT_CLIENTD_INVITE_CODE` environment variable."));
    }

    let app = match cli.mode {
        Mode::Rest => Router::new()
            .nest("/v2", fedimint_v2_rest())
            .with_state(state)
            .layer(ValidateRequestHeaderLayer::bearer(&cli.password)),
        Mode::Ws => Router::new()
            .route("/ws", get(websocket_handler))
            .with_state(state)
            .layer(ValidateRequestHeaderLayer::bearer(&cli.password)),
    };
    info!("Starting server in {:?} mode", cli.mode);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
        // allow auth header
        .allow_headers(Any);

    let metrics = HttpMetricsLayerBuilder::new()
        .with_service_name("fedimint-clientd".to_string())
        .build();

    let app = app
        .layer(cors)
        .layer(TraceLayer::new_for_http()) // tracing requests
        // no traces for routes bellow
        .route("/health", get(|| async { "Server is up and running!" })) // for health check
        // metrics for all routes above
        .merge(metrics.routes())
        .layer(metrics);

    let listener = tokio::net::TcpListener::bind(cli.addr.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to address, should be a valid address and port like 127.0.0.1:3333: {e}"))?;
    info!("fedimint-clientd Listening on {}", &cli.addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {e}"))?;

    Ok(())
}

/// Implements Fedimint V0.2 API Route matching against CLI commands:
/// - `/fedimint/v2/admin/backup`: Upload the (encrypted) snapshot of mint notes
///   to federation.
/// - `/fedimint/v2/admin/discover-version`: Discover the common api version to
///   use to communicate with the federation.
/// - `/fedimint/v2/admin/info`: Display wallet info (holdings, tiers).
/// - `/fedimint/v2/admin/join`: Join a federation with an invite code.
/// - `/fedimint/v2/admin/restore`: Restore the previously created backup of
///   mint notes (with `backup` command).
/// - `/fedimint/v2/admin/list-operations`: List operations.
/// - `/fedimint/v2/admin/module`: Call a module subcommand.
/// - `/fedimint/v2/admin/config`: Returns the client config.
///
/// Mint related commands:
/// - `/fedimint/v2/mint/reissue`: Reissue notes received from a third party to
///   avoid double spends.
/// - `/fedimint/v2/mint/spend`: Prepare notes to send to a third party as a
///   payment.
/// - `/fedimint/v2/mint/validate`: Verifies the signatures of e-cash notes, but
///   *not* if they have been spent already.
/// - `/fedimint/v2/mint/split`: Splits a string containing multiple e-cash
///   notes (e.g. from the `spend` command) into ones that contain exactly one.
/// - `/fedimint/v2/mint/combine`: Combines two or more serialized e-cash notes
///   strings.
///
/// Lightning network related commands:
/// - `/fedimint/v2/ln/invoice`: Create a lightning invoice to receive payment
///   via gateway.
/// - `/fedimint/v2/ln/invoice-external-pubkey-tweaked`: Create a lightning
///   invoice to receive payment via gateway with external pubkey.
/// - `/fedimint/v2/ln/await-invoice`: Wait for incoming invoice to be paid.
/// - `/fedimint/v2/ln/claim-external-receive-tweaked`: Claim an external
///   receive.
/// - `/fedimint/v2/ln/pay`: Pay a lightning invoice or lnurl via a gateway.
/// - `/fedimint/v2/ln/await-pay`: Wait for a lightning payment to complete.
/// - `/fedimint/v2/ln/list-gateways`: List registered gateways.
/// - `/fedimint/v2/ln/switch-gateway`: Switch active gateway.
///
/// Onchain related commands:
/// - `/fedimint/v2/onchain/deposit-address`: Generate a new deposit address,
///   funds sent to it can later be claimed.
/// - `/fedimint/v2/onchain/await-deposit`: Wait for deposit on previously
///   generated address.
/// - `/fedimint/v2/onchain/withdraw`: Withdraw funds from the federation.
fn fedimint_v2_rest() -> Router<AppState> {
    let mint_router = Router::new()
        .route("/decode-notes", post(mint::decode_notes::handle_rest))
        .route("/encode-notes", post(mint::encode_notes::handle_rest))
        .route("/reissue", post(mint::reissue::handle_rest))
        .route("/spend", post(mint::spend::handle_rest))
        .route("/validate", post(mint::validate::handle_rest))
        .route("/split", post(mint::split::handle_rest))
        .route("/combine", post(mint::combine::handle_rest));

    let ln_router = Router::new()
        .route("/invoice", post(ln::invoice::handle_rest))
        .route(
            "/invoice-external-pubkey-tweaked",
            post(ln::invoice_external_pubkey_tweaked::handle_rest),
        )
        .route("/await-invoice", post(ln::await_invoice::handle_rest))
        .route(
            "/claim-external-receive-tweaked",
            post(ln::claim_external_receive_tweaked::handle_rest),
        )
        .route("/pay", post(ln::pay::handle_rest))
        .route("/list-gateways", post(ln::list_gateways::handle_rest));

    let onchain_router = Router::new()
        .route(
            "/deposit-address",
            post(onchain::deposit_address::handle_rest),
        )
        .route("/await-deposit", post(onchain::await_deposit::handle_rest))
        .route("/withdraw", post(onchain::withdraw::handle_rest));

    let admin_router = Router::new()
        .route("/backup", post(admin::backup::handle_rest))
        .route(
            "/discover-version",
            post(admin::discover_version::handle_rest),
        )
        .route("/federation-ids", get(admin::federation_ids::handle_rest))
        .route("/info", get(admin::info::handle_rest))
        .route("/join", post(admin::join::handle_rest))
        .route("/restore", post(admin::restore::handle_rest))
        // .route("/printsecret", get(handle_printsecret)) TODO: should I expose this
        // under admin?
        .route(
            "/list-operations",
            post(admin::list_operations::handle_rest),
        )
        .route("/module", post(admin::module::handle_rest))
        .route("/config", get(admin::config::handle_rest));

    Router::new()
        .nest("/admin", admin_router)
        .nest("/mint", mint_router)
        .nest("/ln", ln_router)
        .nest("/onchain", onchain_router)
}
