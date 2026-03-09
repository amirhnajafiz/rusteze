use std::sync::Arc;
use tokio::sync::Mutex;

use crate::memcache::MemCache;

// Worker function to periodically clean up expired keys from the memcache.
pub async fn worker_memcache_cleanup(seconds: u64, memcache: Arc<Mutex<MemCache>>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(seconds));

    loop {
        interval.tick().await;
        let mut cache = memcache.lock().await;
        cache.cleanup();
        print!("{} expired keys cleaned up from memcache\n", seconds);
    }
}
