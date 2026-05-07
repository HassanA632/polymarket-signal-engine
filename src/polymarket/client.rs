use anyhow::Result;

use crate::polymarket::types::Event;

const GAMMA_API_BASE_URL: &str = "https://gamma-api.polymarket.com";

pub struct PolymarketClient {
    http: reqwest::Client,
}

impl PolymarketClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn fetch_active_events(&self, limit: u32) -> Result<Vec<Event>> {
        let url = format!(
            "{}/events?active=true&closed=false&order=volume&ascending=false&limit={}",
            GAMMA_API_BASE_URL, limit
        );

        let events = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Event>>()
            .await?;

        Ok(events)
    }
}
