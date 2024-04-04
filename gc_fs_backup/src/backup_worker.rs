use std::path::PathBuf;

use clap::Parser;
use gc_fs_backup::GcFsBackupBuilder;

#[derive(Parser)]
#[clap(version = "1.0", author = "Kody Low <kodylow7@gmail.com>")]
struct Cli {
    /// Sets the database path to backup
    #[clap(long, value_name = "DB_PATH", env = "DB_PATH", required = true)]
    db_path: PathBuf,

    /// Sets the bucket ID for the backup
    #[clap(long, value_name = "BUCKET_ID", env = "BUCKET_ID", required = true)]
    bucket_id: String,

    /// Sets the credentials for accessing the storage
    #[clap(long, value_name = "CREDENTIAL", env = "CREDENTIAL", required = true)]
    credential: String,

    /// Sets the name for the backup
    #[clap(long, value_name = "BACKUP_NAME", env = "BACKUP_NAME")]
    backup_name: String,
}

#[tokio::main]
async fn main() {
    let cli: Cli = Cli::parse();

    let backup = GcFsBackupBuilder::new()
        .db_path(cli.db_path.into())
        .bucket_id(cli.bucket_id.into())
        .credential(cli.credential.into())
        .backup_name(cli.backup_name.into())
        .build();
    backup.backup_directory_contents().await.unwrap();
}
