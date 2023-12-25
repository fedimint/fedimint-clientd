use fedimint_client::derivable_secret::DerivableSecret;
use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
use fedimint_core::api::InviteCode;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config =
        Config::from_env().expect("Failed to load config from environment");
}

pub struct Config {
    pub host: String,
    pub port: u16,
    pub invite_code: InviteCode,
    pub root_secret: DerivableSecret,
    pub fm_db_path: PathBuf,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok();

        let host = env::var("HOST").unwrap_or("localhost".to_string());

        let port = env::var("PORT").unwrap_or("3000".to_string());
        let port = u16::from_str(&port).expect("Invalid port");

        let fm_db_path = env::var("FM_DB_PATH").expect("FM_DB_PATH must be set");
        let fm_db_path = PathBuf::from_str(&fm_db_path).expect("Invalid fm db path");

        let invite_code =
            env::var("FEDERATION_INVITE_CODE").expect("FEDERATION_INVITE_CODE must be set");
        let invite_code = InviteCode::from_str(&invite_code).expect("Invalid invite code");

        let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let root_secret = create_root_secret(secret);

        let password = env::var("PASSWORD").expect("PASSWORD must be set");

        info!("Loaded config");

        Ok(Self {
            host,
            port,
            invite_code,
            root_secret,
            fm_db_path,
            password,
        })
    }
}

fn create_root_secret(secret: String) -> DerivableSecret {
    let secret_bytes = secret.as_bytes();
    assert_eq!(secret_bytes.len(), 64, "SECRET_KEY must be 64 bytes long");
    let mut secret_array = [0; 64];
    secret_array.copy_from_slice(secret_bytes);
    PlainRootSecretStrategy::to_root_secret(&secret_array)
}
