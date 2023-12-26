use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};

// Helper function to convert SystemTime to u64
pub fn system_time_to_u64(time: SystemTime) -> Result<u64> {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs()), // Converts to seconds
        Err(_) => Err(anyhow!("some error")),
    }
}
