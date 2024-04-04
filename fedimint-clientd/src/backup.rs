use std::path::{Path, PathBuf};

use inotify::{EventMask, WatchMask};
use tokio::fs::{read, read_dir};

async fn backup_db(
    db_path: PathBuf,
    bucket_id: &str,
    credential: &str,
) -> Result<(), anyhow::Error> {
    let mut inotify = inotify::Inotify::init()?;
    inotify
        .watches()
        .add(
            db_path.as_path(),
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
        )
        .expect("Error adding watch");

    let mut buffer = [0; 1024];

    let bucket = google_cloud::storage::Client::new(credential)
        .await?
        .bucket(bucket_id)
        .await?;

    loop {
        let events = inotify.read_events(&mut buffer)?;
        for event in events {
            if event
                .mask
                .contains(EventMask::CLOSE_WRITE | EventMask::CREATE)
            {
                backup_directory_contents(&db_path, bucket.clone()).await?;
            }
        }
    }
}

async fn backup_directory_contents(
    dir_path: &Path,
    bucket: google_cloud::storage::Bucket,
) -> Result<(), anyhow::Error> {
    let mut entries = read_dir(dir_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            backup_directory_contents(&path, bucket).await?;
        } else {
            let file_contents = read(&path).await?;
            let object_name = path
                .strip_prefix(dir_path)?
                .to_str()
                .ok_or_else(|| anyhow::Error::msg("Path conversion error"))?;
            bucket
                .clone()
                .create_object(object_name, file_contents, "application/octet-stream")
                .await?;
        }
    }
    Ok(())
}
