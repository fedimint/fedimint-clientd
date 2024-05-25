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
    /// Path to FM database
    #[clap(long, env = "FEDIMINT_CLIENTD_DB_PATH", required = true)]
    pub db_path: PathBuf,
    /// Manual secret
    #[clap(long, env = "FEDIMINT_CLIENTD_MANUAL_SECRET", required = false)]
    pub manual_secret: Option<String>,
    /// Location of keys file
    #[clap(long, env = "FEDIMINT_NWC_KEYS_FILE", default_value_t = String::from("keys.json"))]
    pub keys_file: String,
    /// Nostr relay to use
    #[clap(long, env = "FEDIMINT_NWC_RELAYS", default_value_t = String::from("wss://relay.damus.io"))]
    pub relays: String,
    /// Max invoice payment amount, in satoshis
    #[clap(long, env = "FEDIMINT_NWC_MAX_AMOUNT", default_value_t = 100_000)]
    pub max_amount: u64,
    /// Max payment amount per day, in satoshis
    #[clap(long, env = "FEDIMINT_NWC_DAILY_LIMIT", default_value_t = 100_000)]
    pub daily_limit: u64,
}
