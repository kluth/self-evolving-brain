use redis::AsyncCommands;
use anyhow::Result;

pub struct L2Cache {
    client: redis::Client,
}

impl L2Cache {
    pub fn new(connection_str: &str) -> Result<Self> {
        let client = redis::Client::open(connection_str)?;
        Ok(Self { client })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.client.get_async_connection().await?;
        let val: Option<String> = conn.get(key).await?;
        Ok(val)
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: usize) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let _: () = conn.set_ex(key, value, ttl_secs).await?;
        Ok(())
    }
}
