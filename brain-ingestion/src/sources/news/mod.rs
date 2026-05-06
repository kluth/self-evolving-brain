use feed_rs::parser;
use anyhow::Result;
use reqwest::Client;

pub struct RssFetcher {
    client: Client,
}

impl RssFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub async fn fetch_feed(&self, url: &str) -> Result<Vec<feed_rs::model::Entry>> {
        let content = self.client.get(url).send().await?.bytes().await?;
        let feed = parser::parse(&content[..])?;
        Ok(feed.entries)
    }
}
