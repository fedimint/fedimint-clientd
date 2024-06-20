use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Kody Low")]
pub struct Cli {
    /// Federation invite code
    #[clap(long, env = "FEDIMINT_CLIENTD_INVITE_CODE", required = false)]
    pub invite_code: String,
    /// Working directory for all files
    #[clap(long, env = "FEDIMINT_CLIENTD_WORK_DIR", required = true)]
    pub work_dir: PathBuf,
    /// Secret key
    #[clap(long, env = "FEDIMINT_CLIENTD_SECRET_KEY", required = false)]
    pub secret_key: String,
    /// Nostr relay to use
    #[clap(long, env = "FEDIMINT_NWC_RELAYS", default_value_t = String::from("wss://relay.damus.io"))]
    pub relays: String,
    /// Max invoice payment amount, in satoshis
    #[clap(long, env = "FEDIMINT_NWC_MAX_AMOUNT", default_value_t = 100_000)]
    pub max_amount: u64,
    /// Max payment amount per day, in satoshis
    #[clap(long, env = "FEDIMINT_NWC_DAILY_LIMIT", default_value_t = 100_000)]
    pub daily_limit: u64,
    /// Rate limit for payments, in seconds
    #[clap(long, env = "FEDIMINT_NWC_RATE_LIMIT_SECS", default_value_t = 86_400)]
    pub rate_limit_secs: u64,
}
