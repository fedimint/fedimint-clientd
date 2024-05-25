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
    #[clap(default_value_t = String::from("keys.json"), long)]
    pub keys_file: String,
    #[clap(long)]
    /// Nostr relay to use
    pub relay: String,
    /// Max invoice payment amount, in satoshis
    #[clap(default_value_t = 100_000, long)]
    pub max_amount: u64,
    /// Max payment amount per day, in satoshis
    #[clap(default_value_t = 100_000, long)]
    pub daily_limit: u64,
}
