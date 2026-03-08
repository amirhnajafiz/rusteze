pub struct MemCache {
    cache: std::collections::HashMap<String, String>,
    ttl: std::collections::HashMap<String, std::time::Instant>,
}

impl MemCache {
    pub fn new() -> Self {
        MemCache {
            cache: std::collections::HashMap::new(),
            ttl: std::collections::HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, ttl_seconds: Option<u64>) {
        self.cache.insert(key.clone(), value);

        if let Some(seconds) = ttl_seconds {
            self.ttl.insert(key, std::time::Instant::now() + std::time::Duration::from_secs(seconds));
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).cloned()
    }

    pub fn cleanup(&mut self) {
        let now = std::time::Instant::now();
        let expired_keys: Vec<String> = self.ttl.iter()
            .filter(|(_, expiry)| *expiry <= &now)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.remove(&key);
            self.ttl.remove(&key);
        }
    }
}
