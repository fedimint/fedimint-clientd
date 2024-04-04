use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use gc_fs_backup::{BackupTrigger, GcFsBackupBuilder};

use crate::Cli;

// Helper function to convert SystemTime to u64
pub fn system_time_to_u64(time: SystemTime) -> Result<u64> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs()), // Converts to seconds
        Err(_) => Err(anyhow!("some error")),
    }
}

pub async fn start_backup_daemon(cli: &Cli) -> Result<(), anyhow::Error> {
    if let Some(bucket_id) = cli.google_cloud_backup_bucket_id.clone() {
        if let Some(credential) = cli.google_cloud_backup_credentials.clone() {
            if let Some(name) = cli.google_cloud_backup_name.clone() {
                if let (Some(interval_type), Some(interval_value)) = (
                    cli.google_cloud_backup_interval_type.as_ref(),
                    cli.google_cloud_backup_interval_value,
                ) {
                    let trigger = match interval_type.as_str() {
                        "seconds" => BackupTrigger::Interval(Duration::from_secs(interval_value)),
                        "minutes" => {
                            BackupTrigger::Interval(Duration::from_secs(interval_value * 60))
                        }
                        "hours" => {
                            BackupTrigger::Interval(Duration::from_secs(interval_value * 3600))
                        }
                        "days" => {
                            BackupTrigger::Interval(Duration::from_secs(interval_value * 86400))
                        }
                        _ => BackupTrigger::default(),
                    };
                    let backup = GcFsBackupBuilder::new()
                        .db_path(cli.db_path.clone())
                        .bucket_id(bucket_id)
                        .credential(credential)
                        .backup_name(name)
                        .backup_trigger(trigger)
                        .build();
                    backup.watch_and_backup().await;

                    return Ok(());
                }
            }
        }
    }

    Err(anyhow!("Backup configuration is incomplete. To use Google Cloud Backup, you must provide a bucket ID, credentials, and name."))
}
