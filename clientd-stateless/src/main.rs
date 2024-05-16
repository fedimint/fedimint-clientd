use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use axum::http::Method;
use multimint::fedimint_core::api::InviteCode;
use router::{check, info, keys, keysets, melt, mint, swap};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod cashu;
mod error;
pub mod router;
mod state;
mod utils;

use axum::routing::{get, post};
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use clap::{Parser, Subcommand};
use state::AppState;

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

    /// Manual secret
    #[clap(long, env = "FEDIMINT_CLIENTD_MANUAL_SECRET", required = false)]
    manual_secret: Option<String>,
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

    let app = Router::new().nest("/v1", cashu_v1_rest()).with_state(state);
    info!("Starting stateless server");

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

    let listener = tokio::net::TcpListener::bind(format!("{}", &cli.addr))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to address, should be a valid address and port like 127.0.0.1:3333: {e}"))?;
    info!("fedimint-clientd Listening on {}", &cli.addr);
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {e}"))?;

    Ok(())
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
/// - `/cashu/v1/melt/{method}`
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
fn cashu_v1_rest() -> Router<AppState> {
    Router::new()
        .route("/keys", get(keys::handle_keys))
        .route("/keys/:keyset_id", get(keys::handle_keys_keyset_id))
        .route("/keysets", get(keysets::handle_keysets))
        .route("/swap", post(swap::handle_swap))
        .route("/mint/quote/:method", get(mint::quote::handle_method))
        .route(
            "/mint/quote/:method/:quote_id",
            get(mint::quote::handle_method_quote_id),
        )
        .route("/mint/:method", post(mint::method::handle_method))
        .route("/melt/quote/:method", get(melt::quote::handle_method))
        .route(
            "/melt/quote/:method/:quote_id",
            get(melt::quote::handle_method_quote_id),
        )
        .route("/melt/:method", post(melt::method::handle_method))
        .route("/info", get(info::handle_info))
        .route("/check", post(check::handle_check))
}
