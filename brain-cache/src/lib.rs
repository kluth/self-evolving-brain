pub mod l1_moka;
pub mod l2_redis;

use anyhow::Result;
use l1_moka::L1Cache;
use l2_redis::L2Cache;

pub struct HybridCache {
    l1: L1Cache,
    l2: L2Cache,
}

impl HybridCache {
    pub fn new(l2_url: &str) -> Result<Self> {
        Ok(Self {
            l1: L1Cache::new(10_000, 3600), // 10k items, 1h TTL
            l2: L2Cache::new(l2_url)?,
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        // Try L1
        if let Some(val) = self.l1.get(key).await {
            #[cfg(debug_assertions)]
            println!("Cache L1 HIT: {}", key);
            return Ok(Some(val));
        }

        // Try L2
        if let Ok(Some(val)) = self.l2.get(key).await {
            #[cfg(debug_assertions)]
            println!("Cache L2 HIT: {}", key);
            // Populate L1
            self.l1.set(key.to_string(), val.clone()).await;
            return Ok(Some(val));
        }

        #[cfg(debug_assertions)]
        println!("Cache MISS: {}", key);
        Ok(None)
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: usize) -> Result<()> {
        #[cfg(debug_assertions)]
        println!("Cache SET: {}", key);
        self.l1.set(key.to_string(), value.to_string()).await;
        self.l2.set(key, value, ttl_secs).await?;
        Ok(())
    }
}
