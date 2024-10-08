use std::future::ready;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use anyhow::Result;
use axum::extract::{MatchedPath, Request};
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use futures::future::TryFutureExt;
use futures::try_join;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
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
    #[clap(long, env = "FEDIMINT_CLIENTD_DB_PATH", required = true)]
    db_path: PathBuf,

    /// Password
    #[clap(long, env = "FEDIMINT_CLIENTD_PASSWORD", required = true)]
    password: String,

    /// Addr
    #[clap(long, env = "FEDIMINT_CLIENTD_ADDR", required = true)]
    addr: String,

    /// Prometheus addr
    #[clap(long, env = "PROMETHEUS_ADDR", default_value = "127.0.0.1:3001")]
    prometheus_addr: String,

    /// Manual secret
    #[clap(long, env = "FEDIMINT_CLIENTD_MANUAL_SECRET", required = false)]
    manual_secret: Option<String>,

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

    let mut state = AppState::new(cli.db_path).await?;

    let manual_secret = match cli.manual_secret {
        Some(secret) => Some(secret),
        None => match std::env::var("FEDIMINT_CLIENTD_MANUAL_SECRET") {
            Ok(secret) => Some(secret),
            Err(_) => None,
        },
    };

    match InviteCode::from_str(&cli.invite_code) {
        Ok(invite_code) => {
            let federation_id = state
                .multimint
                .register_new(invite_code, manual_secret)
                .await?;
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

    let main_server = start_main_server(&cli.addr, &cli.password, cli.mode, state)
        .map_err(|e| e.context("main server has failed"));
    let metrics_server = start_metrics_server(&cli.prometheus_addr)
        .map_err(|e| e.context("metrics server has failed"));

    try_join!(main_server, metrics_server)?;
    Ok(())
}

async fn start_main_server(
    addr: &str,
    password: &str,
    mode: Mode,
    state: AppState,
) -> anyhow::Result<()> {
    let app = match mode {
        Mode::Rest => Router::new()
            .nest("/v2", fedimint_v2_rest())
            .with_state(state)
            .layer(ValidateRequestHeaderLayer::bearer(password)),
        Mode::Ws => Router::new()
            .route("/ws", get(websocket_handler))
            .with_state(state)
            .layer(ValidateRequestHeaderLayer::bearer(password)),
    };
    info!("Starting server in {mode:?} mode");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = app
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .route("/health", get(|| async { "Server is up and running!" }))
        .route_layer(middleware::from_fn(track_metrics));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("fedimint-clientd listening on {addr:?}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn start_metrics_server(bind: &str) -> anyhow::Result<()> {
    let app = metrics_app()?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("Prometheus listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

fn metrics_app() -> anyhow::Result<Router> {
    let recorder_handle = setup_metrics_recorder()?;
    Ok(Router::new().route("/metrics", get(move || ready(recorder_handle.render()))))
}

fn setup_metrics_recorder() -> anyhow::Result<PrometheusHandle> {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    Ok(PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )?
        .install_recorder()?)
}

async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);

    response
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
