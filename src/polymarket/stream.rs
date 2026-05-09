use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::polymarket::state::TokenMarketState;
use crate::polymarket::ws_types::{
    BestBidAsk, LastTradePrice, OrderBookSnapshot, PriceChange, TickSizeChange,
};

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

    let mut state = TokenMarketState::new(token_id);

    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                handle_market_message(&text, &mut state);
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

fn handle_market_message(raw_message: &str, state: &mut TokenMarketState) {
    let value = match serde_json::from_str::<Value>(raw_message) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(%error, raw_message, "Failed to parse WebSocket JSON");
            return;
        }
    };

    match value {
        Value::Array(messages) => {
            for message in messages {
                handle_market_message_value(message, state);
            }
        }
        Value::Object(_) => {
            handle_market_message_value(value, state);
        }
        Value::String(message) => {
            tracing::debug!(message, "Received WebSocket text control message");
        }
        other => {
            tracing::debug!(?other, "Ignoring unsupported WebSocket payload");
        }
    }
}

fn handle_market_message_value(value: Value, state: &mut TokenMarketState) {
    let event_type = match value.get("event_type").and_then(Value::as_str) {
        Some(event_type) => event_type,
        None => {
            tracing::debug!(?value, "Ignoring WebSocket message without event_type");
            return;
        }
    };

    match event_type {
        "book" => match serde_json::from_value::<OrderBookSnapshot>(value) {
            Ok(book) => {
                state.apply_book_snapshot(&book);

                println!(
                    "BOOK asset={} bids={} asks={} timestamp={}",
                    book.asset_id,
                    book.bids.len(),
                    book.asks.len(),
                    book.timestamp
                );

                state.display_summary();
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse book message");
            }
        },
        "price_change" => match serde_json::from_value::<PriceChange>(value) {
            Ok(price_change) => {
                state.apply_price_change(&price_change);

                println!(
                    "PRICE_CHANGE market={} changes={} timestamp={}",
                    price_change.market,
                    price_change.changes.len(),
                    price_change.timestamp
                );

                for change in price_change.changes {
                    println!(
                        "  {} {} price={} size={} best_bid={:?} best_ask={:?}",
                        change.asset_id,
                        change.side,
                        change.price,
                        change.size,
                        change.best_bid,
                        change.best_ask
                    );
                }

                state.display_summary();
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse price_change message");
            }
        },
        "last_trade_price" => match serde_json::from_value::<LastTradePrice>(value) {
            Ok(trade) => {
                state.apply_last_trade_price(&trade);

                println!(
                    "LAST_TRADE asset={} side={} price={} size={} timestamp={}",
                    trade.asset_id, trade.side, trade.price, trade.size, trade.timestamp
                );

                state.display_summary();
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse last_trade_price message");
            }
        },
        "best_bid_ask" => match serde_json::from_value::<BestBidAsk>(value) {
            Ok(best_bid_ask) => {
                state.apply_best_bid_ask(&best_bid_ask);

                println!(
                    "BEST_BID_ASK asset={} bid={} ask={} spread={} timestamp={}",
                    best_bid_ask.asset_id,
                    best_bid_ask.best_bid,
                    best_bid_ask.best_ask,
                    best_bid_ask.spread,
                    best_bid_ask.timestamp
                );

                state.display_summary();
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse best_bid_ask message");
            }
        },
        "tick_size_change" => match serde_json::from_value::<TickSizeChange>(value) {
            Ok(tick_size_change) => {
                println!(
                    "TICK_SIZE_CHANGE asset={} old={} new={} timestamp={}",
                    tick_size_change.asset_id,
                    tick_size_change.old_tick_size,
                    tick_size_change.new_tick_size,
                    tick_size_change.timestamp
                );
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse tick_size_change message");
            }
        },
        other => {
            tracing::debug!(event_type = other, "Ignoring unsupported market event");
        }
    }
}
