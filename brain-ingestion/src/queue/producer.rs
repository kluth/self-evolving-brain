use redis::AsyncCommands;
use anyhow::Result;
use super::QueueItem;

pub struct QueueProducer {
    client: redis::Client,
}

impl QueueProducer {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }

    pub async fn push(&self, stream: &str, item: &QueueItem) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let data = serde_json::to_string(item)?;
        let _: () = conn.xadd(stream, "*", &[("data", data)]).await?;
        Ok(())
    }
}
