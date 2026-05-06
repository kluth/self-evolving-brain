use anyhow::{Result, anyhow};
use google_gmail1::{api::Message, Gmail};
use google_calendar3::{api::Event, CalendarHub};
use google_apis_common::oauth2::{read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use hyper::Client;
use hyper_rustls::HttpsConnectorBuilder;
use std::path::Path;

pub struct GoogleAdapter {
    pub hub_gmail: Option<Gmail<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>,
    pub hub_calendar: Option<CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>>,
}

impl GoogleAdapter {
    pub async fn new(client_secret_path: &str) -> Result<Self> {
        if !Path::new(client_secret_path).exists() {
            println!("Google client secret not found at {}, skipping Google integration.", client_secret_path);
            return Ok(Self { hub_gmail: None, hub_calendar: None });
        }

        let secret = read_application_secret(client_secret_path).await?;
        
        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::Interactive,
        ).persist_tokens_to_disk("tokens.json")
         .build()
         .await?;

        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();

        let client = Client::builder().build(https.clone());

        let hub_gmail = Gmail::new(client.clone(), auth.clone());
        let hub_calendar = CalendarHub::new(client, auth);

        Ok(Self {
            hub_gmail: Some(hub_gmail),
            hub_calendar: Some(hub_calendar),
        })
    }

    pub async fn fetch_messages(&self) -> Result<Vec<Message>> {
        let hub = self.hub_gmail.as_ref().ok_or_else(|| anyhow!("Gmail not initialized"))?;
        let result = hub.users().messages_list("me").doit().await?;
        Ok(result.1.messages.unwrap_or_default())
    }

    pub async fn fetch_calendar_events(&self) -> Result<Vec<Event>> {
        let hub = self.hub_calendar.as_ref().ok_or_else(|| anyhow!("Calendar not initialized"))?;
        let result = hub.events().list("primary").doit().await?;
        Ok(result.1.items.unwrap_or_default())
    }
}
