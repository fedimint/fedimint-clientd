use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use flate2::write::GzEncoder;
use flate2::Compression;
use inotify::{EventMask, Inotify, WatchMask};
use tar::Builder;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub enum BackupTrigger {
    Interval(Duration),
    EventMasks(Vec<EventMask>),
}

impl Default for BackupTrigger {
    fn default() -> Self {
        let default_masks = vec![EventMask::CLOSE_WRITE | EventMask::CREATE];
        Self::EventMasks(default_masks)
    }
}

pub struct GcFsBackup {
    db_path: PathBuf,
    backup_name: Option<String>,
    bucket_id: String,
    credential: String,
    backup_trigger: BackupTrigger,
}

impl GcFsBackup {
    pub async fn backup_directory_contents(&self) -> Result<(), anyhow::Error> {
        let dir_path = &self.db_path;
        let bucket_id = &self.bucket_id;
        let credential = &self.credential;

        let tarball_data = Vec::new();
        let gz_encoder = GzEncoder::new(tarball_data, Compression::default());
        let mut ar = Builder::new(gz_encoder);
        let backup_name = self
            .backup_name
            .clone()
            .unwrap_or("fedimint_db_backup".to_string());
        ar.append_dir_all(backup_name, dir_path)?;

        let gz_encoder = ar.into_inner()?;
        let compressed_tarball_bytes = gz_encoder.finish()?;

        let client = Arc::new(Mutex::new(
            google_cloud::storage::Client::new(credential).await?,
        ));
        let mut bucket = client.lock().await.bucket(bucket_id).await?;
        bucket
            .create_object(
                "fedimint_db_backup.tar.gz",
                compressed_tarball_bytes,
                "application/gzip",
            )
            .await?;

        Ok(())
    }

    pub async fn watch_and_backup(&self) {
        match &self.backup_trigger {
            BackupTrigger::Interval(duration) => loop {
                sleep(*duration).await;
                self.backup_directory_contents()
                    .await
                    .expect("Failed to backup directory contents");
            },
            BackupTrigger::EventMasks(masks) => {
                let db_path = self.db_path.clone();
                let mut inotify = Inotify::init().expect("Failed to initialize inotify");
                let watch_mask = masks.iter().fold(WatchMask::empty(), |acc, mask| {
                    acc | WatchMask::from_bits_truncate(mask.bits())
                });
                inotify
                    .watches()
                    .add(&db_path, watch_mask)
                    .expect("Failed to add watch");

                let mut buffer = [0; 1024];
                loop {
                    let events = inotify
                        .read_events_blocking(&mut buffer)
                        .expect("Failed to read inotify events");
                    for event in events {
                        if masks.iter().any(|mask| event.mask.contains(*mask)) {
                            self.backup_directory_contents()
                                .await
                                .expect("Failed to backup directory contents");
                        }
                    }
                }
            }
        }
    }
}

pub struct GcFsBackupBuilder {
    pub db_path: PathBuf,
    pub bucket_id: String,
    pub credential: String,
    pub backup_name: Option<String>,
    pub backup_trigger: BackupTrigger,
}

impl GcFsBackupBuilder {
    pub fn new() -> Self {
        Self {
            db_path: PathBuf::new(),
            backup_name: None,
            bucket_id: String::new(),
            credential: String::new(),
            backup_trigger: BackupTrigger::default(),
        }
    }

    pub fn db_path(mut self, db_path: PathBuf) -> Self {
        self.db_path = db_path;
        self
    }

    pub fn backup_name(mut self, backup_name: String) -> Self {
        self.backup_name = Some(backup_name);
        self
    }

    pub fn bucket_id(mut self, bucket_id: String) -> Self {
        self.bucket_id = bucket_id;
        self
    }

    pub fn credential(mut self, credential: String) -> Self {
        self.credential = credential;
        self
    }

    pub fn backup_trigger(mut self, trigger: BackupTrigger) -> Self {
        self.backup_trigger = trigger;
        self
    }

    pub fn build(self) -> GcFsBackup {
        GcFsBackup {
            db_path: self.db_path,
            credential: self.credential,
            bucket_id: self.bucket_id,
            backup_name: self.backup_name,
            backup_trigger: self.backup_trigger,
        }
    }
}
