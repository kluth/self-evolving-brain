use moka::future::Cache;
use std::time::Duration;

pub struct L1Cache {
    cache: Cache<String, String>,
}

impl L1Cache {
    pub fn new(capacity: u64, ttl_secs: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(capacity)
                .time_to_live(Duration::from_secs(ttl_secs))
                .build(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).await
    }

    pub async fn set(&self, key: String, value: String) {
        self.cache.insert(key, value).await;
    }
}
