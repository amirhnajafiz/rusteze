use std::sync::Arc;
use tokio::sync::Mutex;

use crate::memcache::MemCache;
use tracing::info;

// Worker function to periodically clean up expired keys from the memcache.
pub async fn worker_memcache_cleanup(seconds: u64, memcache: Arc<Mutex<MemCache>>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(seconds));
    info!("memcache cleanup worker started, running every {} seconds", seconds);

    loop {
        interval.tick().await;

        let mut cache = memcache.lock().await;
        cache.cleanup();

        info!("expired keys cleaned up from memcache");
    }
}

pub async fn worker_snapshot(seconds: u64, path: String, memcache: Arc<Mutex<MemCache>>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(seconds));
    info!("snapshot worker started, running every {} seconds", seconds);

    loop {
        interval.tick().await;

        let cache = memcache.lock().await;
        let snapshot = cache.export_cache();

        // save the snapshot to a file in path directory with a timestamp in the filename
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("{}/snapshot_{}.json", path, timestamp);
        match std::fs::write(&filename, serde_json::to_string(&snapshot).unwrap()) {
            Ok(_) => info!("snapshot saved to {}", filename),
            Err(e) => info!("failed to save snapshot: {}", e),
        }

        info!("snapshot taken and save to file {}", filename);
    }
}