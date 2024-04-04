use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use axum::http::Method;
use fedimint_core::api::InviteCode;
use router::ws::websocket_handler;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod error;
mod router;
mod state;
mod utils;

use axum::routing::{get, post};
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use clap::{Parser, Subcommand, ValueEnum};
use router::handlers::*;
use state::AppState;
// use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::utils::start_backup_daemon;

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Mode {
    Rest,
    Ws,
    Cashu,
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

    /// Manual secret
    #[clap(long, env = "FEDIMINT_CLIENTD_MANUAL_SECRET", required = false)]
    manual_secret: Option<String>,

    /// Mode: ws, rest
    #[clap(long, env = "FEDIMINT_CLIENTD_MODE", default_value = "rest")]
    mode: Mode,

    /// Enable Google Cloud backup
    #[clap(long, env = "USE_GOOGLE_CLOUD_BACKUP")]
    use_google_cloud_backup: bool,

    /// Google Cloud Storage Bucket ID
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_BUCKET_ID",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_bucket_id: Option<String>,

    /// Google Cloud Storage Credentials
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_CREDENTIALS",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_credentials: Option<String>,

    /// Google Cloud Storage Backup Name
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_NAME",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_name: Option<String>,

    /// Google Cloud Storage Backup Trigger
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_TRIGGER",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_trigger: Option<String>,

    /// Google Cloud Backup Interval Type (seconds, minutes, hours, days)
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_INTERVAL_TYPE",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_interval_type: Option<String>,

    /// Google Cloud Backup Interval Value
    #[clap(
        long,
        env = "GOOGLE_CLOUD_BACKUP_INTERVAL_VALUE",
        required_if_eq("use_google_cloud_backup", "true")
    )]
    google_cloud_backup_interval_value: Option<u64>,
}

// const PID_FILE: &str = "/tmp/fedimint_http.pid";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let cli: Cli = Cli::parse();
    start_backup_daemon(&cli).await?;

    let mut state = AppState::new(cli.db_path.clone()).await?;

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
            if cli.mode == Mode::Cashu {
                state.cashu_mint = Some(federation_id);
            }
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
        Mode::Cashu => Router::new()
            .nest("/v1", cashu_v1_rest())
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

    let listener = tokio::net::TcpListener::bind(format!("{}", &cli.addr))
        .await
        .expect(
            "Failed to bind to address, should be a valid address and port like 127.0.0.1:3333",
        );
    info!("fedimint-clientd Listening on {}", &cli.addr);
    axum::serve(listener, app).await.unwrap();

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
/// - `/fedimint/v2/ln/invoice-external-pubkey`: Create a lightning invoice to
///  receive payment via gateway with external pubkey.
/// - `/fedimint/v2/ln/await-invoice`: Wait for incoming invoice to be paid.
/// - `/fedimint/v2/ln/claim-external-receive`: Claim an external receive.
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
        .route(
            "/decode-notes",
            post(fedimint::mint::decode_notes::handle_rest),
        )
        .route(
            "/encode-notes",
            post(fedimint::mint::encode_notes::handle_rest),
        )
        .route("/reissue", post(fedimint::mint::reissue::handle_rest))
        .route("/spend", post(fedimint::mint::spend::handle_rest))
        .route("/validate", post(fedimint::mint::validate::handle_rest))
        .route("/split", post(fedimint::mint::split::handle_rest))
        .route("/combine", post(fedimint::mint::combine::handle_rest));

    let ln_router = Router::new()
        .route("/invoice", post(fedimint::ln::invoice::handle_rest))
        .route(
            "/invoice-external-pubkey",
            post(fedimint::ln::invoice_external_pubkey::handle_rest),
        )
        .route(
            "/invoice-external-pubkey-tweaked",
            post(fedimint::ln::invoice_external_pubkey_tweaked::handle_rest),
        )
        .route(
            "/await-invoice",
            post(fedimint::ln::await_invoice::handle_rest),
        )
        .route(
            "/claim-external-receive",
            post(fedimint::ln::claim_external_receive::handle_rest),
        )
        .route(
            "/claim-external-receive-tweaked",
            post(fedimint::ln::claim_external_receive_tweaked::handle_rest),
        )
        .route("/pay", post(fedimint::ln::pay::handle_rest))
        .route(
            "/list-gateways",
            post(fedimint::ln::list_gateways::handle_rest),
        );

    let wallet_router = Router::new()
        .route(
            "/deposit-address",
            post(fedimint::onchain::deposit_address::handle_rest),
        )
        .route(
            "/await-deposit",
            post(fedimint::onchain::await_deposit::handle_rest),
        )
        .route("/withdraw", post(fedimint::onchain::withdraw::handle_rest));

    let admin_router = Router::new()
        .route("/backup", post(fedimint::admin::backup::handle_rest))
        .route(
            "/discover-version",
            post(fedimint::admin::discover_version::handle_rest),
        )
        .route(
            "/federation-ids",
            get(fedimint::admin::federation_ids::handle_rest),
        )
        .route("/info", get(fedimint::admin::info::handle_rest))
        .route("/join", post(fedimint::admin::join::handle_rest))
        .route("/restore", post(fedimint::admin::restore::handle_rest))
        // .route("/printsecret", get(fedimint::handle_printsecret)) TODO: should I expose this
        // under admin?
        .route(
            "/list-operations",
            post(fedimint::admin::list_operations::handle_rest),
        )
        .route("/module", post(fedimint::admin::module::handle_rest))
        .route("/config", get(fedimint::admin::config::handle_rest));

    Router::new()
        .nest("/admin", admin_router)
        .nest("/mint", mint_router)
        .nest("/ln", ln_router)
        .nest("/wallet", wallet_router)
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
        .route("/keys", get(cashu::keys::handle_keys))
        .route("/keys/:keyset_id", get(cashu::keys::handle_keys_keyset_id))
        .route("/keysets", get(cashu::keysets::handle_keysets))
        .route("/swap", post(cashu::swap::handle_swap))
        .route(
            "/mint/quote/:method",
            get(cashu::mint::quote::handle_method),
        )
        .route(
            "/mint/quote/:method/:quote_id",
            get(cashu::mint::quote::handle_method_quote_id),
        )
        .route("/mint/:method", post(cashu::mint::method::handle_method))
        .route(
            "/melt/quote/:method",
            get(cashu::melt::quote::handle_method),
        )
        .route(
            "/melt/quote/:method/:quote_id",
            get(cashu::melt::quote::handle_method_quote_id),
        )
        .route("/melt/:method", post(cashu::melt::method::handle_method))
        .route("/info", get(cashu::info::handle_info))
        .route("/check", post(cashu::check::handle_check))
}
