use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const POLYMARKET_MARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

pub async fn stream_token(token_id: &str) -> Result<()> {
    tracing::info!(token_id, "Connecting to Polymarket market WebSocket");

    let (ws_stream, _) = connect_async(POLYMARKET_MARKET_WS_URL).await?;
    let (mut write, mut read) = ws_stream.split();

    let subscription = json!({
        "assets_ids": [token_id],
        "type": "market",
        "custom_feature_enabled": true
    });

    write
        .send(Message::Text(subscription.to_string().into()))
        .await?;

    println!("Connected to Polymarket market WebSocket");
    println!("Subscribed to token: {}", token_id);
    println!("Waiting for live market messages...");
    println!();

    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                println!("{}", text);
            }
            Message::Ping(payload) => {
                tracing::debug!("Received ping");
                write.send(Message::Pong(payload)).await?;
            }
            Message::Close(frame) => {
                tracing::warn!(?frame, "WebSocket closed");
                break;
            }
            other => {
                tracing::debug!(?other, "Received non-text WebSocket message");
            }
        }
    }

    Ok(())
}
