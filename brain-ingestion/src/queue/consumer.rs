use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::AsyncCommands;
use anyhow::Result;
use super::QueueItem;

pub struct QueueConsumer {
    client: redis::Client,
}

impl QueueConsumer {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }

    pub async fn consume(&self, stream: &str) -> Result<Vec<(String, QueueItem)>> {
        let mut conn = self.client.get_async_connection().await?;
        let options = StreamReadOptions::default().count(10).block(5000);
        let reply: StreamReadReply = conn.xread_options(&[stream], &["0"], &options).await?;

        let mut results = Vec::new();
        for stream_key in reply.keys {
            for entry in stream_key.ids {
                if let Some(data) = entry.map.get("data") {
                    if let redis::Value::Data(bytes) = data {
                        let item: QueueItem = serde_json::from_slice(bytes)?;
                        results.push((entry.id, item));
                    }
                }
            }
        }
        Ok(results)
    }

    pub async fn ack(&self, stream: &str, id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let _: () = conn.xdel(stream, &[id]).await?; // Simplification: just delete if handled
        Ok(())
    }
}
