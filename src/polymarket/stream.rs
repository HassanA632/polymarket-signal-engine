use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::polymarket::metrics::{LatencyMetrics, SignalMetrics};
use crate::polymarket::paper_trader::{PaperExitReason, PaperTrader};
use crate::polymarket::signal_logger::SignalLogger;
use crate::polymarket::signals::{
    MarketSignal, SignalConfig, SignalOutputMode, display_signal, evaluate_signals,
};
use crate::polymarket::state::TokenMarketState;
use crate::polymarket::ws_types::{
    BestBidAsk, LastTradePrice, OrderBookSnapshot, PriceChange, TickSizeChange,
};

const POLYMARKET_MARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

#[derive(Debug, Clone, Copy)]
pub struct StreamOutputConfig {
    pub show_state: bool,
    pub show_events: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PaperTradingConfig {
    pub enabled: bool,
    pub stake: f64,
    pub take_profit: f64,
    pub stop_loss: f64,
}

pub async fn stream_token(
    token_id: &str,
    signal_config: SignalConfig,
    signal_output_mode: SignalOutputMode,
    signal_log_path: Option<PathBuf>,
    output_config: StreamOutputConfig,
    paper_trading_config: PaperTradingConfig,
) -> Result<()> {
    println!(
        "Signal config: tight_spread_threshold={} min_spread_tightening={} min_price_move={} large_trade_threshold={}",
        signal_config.tight_spread_threshold,
        signal_config.min_spread_tightening,
        signal_config.min_price_move,
        signal_config.large_trade_threshold
    );
    println!("Signal output mode: {:?}", signal_output_mode);
    println!(
        "Stream output: show_state={} show_events={}",
        output_config.show_state, output_config.show_events
    );
    println!(
        "Paper trading: enabled={} stake={} take_profit={} stop_loss={}",
        paper_trading_config.enabled,
        paper_trading_config.stake,
        paper_trading_config.take_profit,
        paper_trading_config.stop_loss
    );
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
    let mut metrics = LatencyMetrics::default();
    let mut signal_metrics = SignalMetrics::default();
    let mut paper_trader = PaperTrader::new(paper_trading_config.stake);

    let mut signal_logger = match signal_log_path {
        Some(path) => {
            println!("Signal logging enabled: {}", path.display());
            Some(SignalLogger::new(path)?)
        }
        None => None,
    };

    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                let started_at = Instant::now();

                let previous_state = state.clone();
                let handled_message = handle_market_message(&text, &mut state, output_config);

                if handled_message {
                    for signal in evaluate_signals(Some(&previous_state), &state, &signal_config) {
                        display_signal(&signal, signal_output_mode);
                        signal_metrics.record(&signal);

                        if paper_trading_config.enabled {
                            maybe_open_paper_position(
                                &mut paper_trader,
                                &state,
                                &signal,
                                &signal_config,
                            );
                        }

                        if let Some(logger) = signal_logger.as_mut() {
                            logger.log(&signal)?;
                        }
                    }

                    if paper_trading_config.enabled {
                        maybe_close_paper_position(&mut paper_trader, &state, paper_trading_config);
                    }

                    let processing_latency = started_at.elapsed();

                    println!("processing_latency={:?}", processing_latency);

                    metrics.record(processing_latency);

                    if metrics.should_report(10) {
                        metrics.display_summary();
                        signal_metrics.display_summary();
                    }

                    println!();
                }
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

fn handle_market_message(
    raw_message: &str,
    state: &mut TokenMarketState,
    output_config: StreamOutputConfig,
) -> bool {
    let value = match serde_json::from_str::<Value>(raw_message) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(%error, raw_message, "Failed to parse WebSocket JSON");
            return false;
        }
    };

    match value {
        Value::Array(messages) => {
            let mut handled_any = false;

            for message in messages {
                if handle_market_message_value(message, state, output_config) {
                    handled_any = true;
                }
            }

            handled_any
        }
        Value::Object(_) => handle_market_message_value(value, state, output_config),
        Value::String(message) => {
            tracing::debug!(message, "Received WebSocket text control message");
            false
        }
        other => {
            tracing::debug!(?other, "Ignoring unsupported WebSocket payload");
            false
        }
    }
}

fn handle_market_message_value(
    value: Value,
    state: &mut TokenMarketState,
    output_config: StreamOutputConfig,
) -> bool {
    let event_type = match value.get("event_type").and_then(Value::as_str) {
        Some(event_type) => event_type,
        None => {
            tracing::debug!(?value, "Ignoring WebSocket message without event_type");
            return false;
        }
    };

    match event_type {
        "book" => match serde_json::from_value::<OrderBookSnapshot>(value) {
            Ok(book) => {
                state.apply_book_snapshot(&book);

                if output_config.show_events {
                    println!(
                        "BOOK asset={} bids={} asks={} timestamp={}",
                        book.asset_id,
                        book.bids.len(),
                        book.asks.len(),
                        book.timestamp
                    );
                }

                if output_config.show_state {
                    state.display_summary();
                }

                true
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse book message");
                false
            }
        },
        "price_change" => match serde_json::from_value::<PriceChange>(value) {
            Ok(price_change) => {
                state.apply_price_change(&price_change);

                if output_config.show_events {
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
                }

                if output_config.show_state {
                    state.display_summary();
                }

                true
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse price_change message");
                false
            }
        },
        "last_trade_price" => match serde_json::from_value::<LastTradePrice>(value) {
            Ok(trade) => {
                state.apply_last_trade_price(&trade);

                if output_config.show_events {
                    println!(
                        "LAST_TRADE asset={} side={} price={} size={} timestamp={}",
                        trade.asset_id, trade.side, trade.price, trade.size, trade.timestamp
                    );
                }

                if output_config.show_state {
                    state.display_summary();
                }

                true
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse last_trade_price message");
                false
            }
        },
        "best_bid_ask" => match serde_json::from_value::<BestBidAsk>(value) {
            Ok(best_bid_ask) => {
                state.apply_best_bid_ask(&best_bid_ask);

                if output_config.show_events {
                    println!(
                        "BEST_BID_ASK asset={} bid={} ask={} spread={} timestamp={}",
                        best_bid_ask.asset_id,
                        best_bid_ask.best_bid,
                        best_bid_ask.best_ask,
                        best_bid_ask.spread,
                        best_bid_ask.timestamp
                    );
                }

                if output_config.show_state {
                    state.display_summary();
                }

                true
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse best_bid_ask message");
                false
            }
        },
        "tick_size_change" => match serde_json::from_value::<TickSizeChange>(value) {
            Ok(tick_size_change) => {
                if output_config.show_events {
                    println!(
                        "TICK_SIZE_CHANGE asset={} old={} new={} timestamp={}",
                        tick_size_change.asset_id,
                        tick_size_change.old_tick_size,
                        tick_size_change.new_tick_size,
                        tick_size_change.timestamp
                    );
                }

                true
            }
            Err(error) => {
                tracing::warn!(%error, "Failed to parse tick_size_change message");
                false
            }
        },
        other => {
            tracing::debug!(event_type = other, "Ignoring unsupported market event");
            false
        }
    }
}

fn maybe_open_paper_position(
    paper_trader: &mut PaperTrader,
    state: &TokenMarketState,
    signal: &MarketSignal,
    signal_config: &SignalConfig,
) {
    if !matches!(signal, MarketSignal::PriceMoveUp { .. }) {
        return;
    }

    if paper_trader.has_open_position() {
        return;
    }

    if !state_has_tight_spread(state, signal_config.tight_spread_threshold) {
        return;
    }

    let Some(entry_price) = state.best_ask.as_deref().and_then(parse_price) else {
        return;
    };

    let opened = paper_trader.open_long(
        state.token_id.clone(),
        entry_price,
        "PriceMoveUp with tight spread",
    );

    if opened {
        println!(
            "PAPER_TRADE Opened side=LONG token={} entry={} stake={} reason=\"PriceMoveUp with tight spread\"",
            shorten_token_id(&state.token_id),
            entry_price,
            paper_trader.stake
        );
    }
}

fn maybe_close_paper_position(
    paper_trader: &mut PaperTrader,
    state: &TokenMarketState,
    paper_trading_config: PaperTradingConfig,
) {
    let Some(exit_price) = state.best_bid.as_deref().and_then(parse_price) else {
        return;
    };

    let Some(closed_trade) = paper_trader.maybe_close_position(
        exit_price,
        paper_trading_config.take_profit,
        paper_trading_config.stop_loss,
    ) else {
        return;
    };

    let exit_reason = match closed_trade.exit_reason {
        PaperExitReason::TakeProfit => "TakeProfit",
        PaperExitReason::StopLoss => "StopLoss",
    };

    println!(
        "PAPER_TRADE Closed side=LONG token={} entry={} exit={} stake={} pnl={:.4} reason={}",
        shorten_token_id(&closed_trade.token_id),
        closed_trade.entry_price,
        closed_trade.exit_price,
        closed_trade.stake,
        closed_trade.pnl,
        exit_reason
    );
}

fn state_has_tight_spread(state: &TokenMarketState, threshold: f64) -> bool {
    state
        .spread
        .as_deref()
        .and_then(parse_price)
        .is_some_and(|spread| spread <= threshold)
}

fn parse_price(value: &str) -> Option<f64> {
    value.parse::<f64>().ok()
}

fn shorten_token_id(token_id: &str) -> String {
    const PREFIX_LEN: usize = 8;
    const SUFFIX_LEN: usize = 6;

    if token_id.len() <= PREFIX_LEN + SUFFIX_LEN {
        return token_id.to_string();
    }

    format!(
        "{}...{}",
        &token_id[..PREFIX_LEN],
        &token_id[token_id.len() - SUFFIX_LEN..]
    )
}
