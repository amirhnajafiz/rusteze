// MemCache implementation for in-memory caching with TTL (Time To Live).
pub struct MemCache {
    cache: std::collections::HashMap<String, String>,
    ttl: std::collections::HashMap<String, std::time::Instant>,
}

// API for the MemCache, including methods to set, get, and clean up expired keys.
impl MemCache {
    pub fn new() -> Self {
        MemCache {
            cache: std::collections::HashMap::new(),
            ttl: std::collections::HashMap::new(),
        }
    }

    // Set a key-value pair in the cache with an optional TTL (Time To Live) in seconds.
    pub fn set(&mut self, key: String, value: String, ttl_seconds: Option<u64>) {
        self.cache.insert(key.clone(), value);

        // if TTL is provided, calculate the expiration time and store it in the ttl map
        if let Some(seconds) = ttl_seconds {
            self.ttl.insert(key, std::time::Instant::now() + std::time::Duration::from_secs(seconds));
        }
    }

    // Get the value associated with a key from the cache. Returns None if the key does not exist or has expired.
    pub fn get(&mut self, key: &str) -> Option<String> {
        // check if the key exists in the cache
        if let Some(value) = self.cache.get(key) {
            // if the key has a TTL, check if it has expired
            if let Some(expiry) = self.ttl.get(key) {
                if *expiry > std::time::Instant::now() {
                    return Some(value.clone());
                } else {
                    // if the key has expired, remove it from the cache and ttl maps
                    self.cache.remove(key);
                    self.ttl.remove(key);
                    return None;
                }
            }

            // If there is no TTL, return the value.
            return Some(value.clone());
        }

        None
    }

    // Clean up expired keys from the cache. This method should be called periodically to ensure that expired keys are removed.
    pub fn cleanup(&mut self) {
        // get the current time and find all keys that have expired
        let now = std::time::Instant::now();
        let expired_keys: Vec<String> = self.ttl.iter()
            .filter(|(_, expiry)| *expiry <= &now)
            .map(|(key, _)| key.clone())
            .collect();

        // remove all expired keys from the cache and ttl maps
        for key in expired_keys {
            self.cache.remove(&key);
            self.ttl.remove(&key);
        }
    }
}
