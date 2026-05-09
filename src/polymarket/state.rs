use crate::polymarket::ws_types::{BestBidAsk, LastTradePrice, OrderBookSnapshot, PriceChange};

#[derive(Debug, Clone)]
pub struct TokenMarketState {
    pub token_id: String,
    pub market_id: Option<String>,
    pub best_bid: Option<String>,
    pub best_ask: Option<String>,
    pub spread: Option<String>,
    pub last_trade_price: Option<String>,
    pub last_trade_size: Option<String>,
    pub last_trade_side: Option<String>,
    pub last_update_timestamp: Option<String>,
    pub bid_levels: usize,
    pub ask_levels: usize,
}

impl TokenMarketState {
    pub fn new(token_id: impl Into<String>) -> Self {
        Self {
            token_id: token_id.into(),
            market_id: None,
            best_bid: None,
            best_ask: None,
            spread: None,
            last_trade_price: None,
            last_trade_size: None,
            last_trade_side: None,
            last_update_timestamp: None,
            bid_levels: 0,
            ask_levels: 0,
        }
    }

    pub fn apply_book_snapshot(&mut self, book: &OrderBookSnapshot) {
        if book.asset_id != self.token_id {
            return;
        }

        self.market_id = Some(book.market.clone());
        self.bid_levels = book.bids.len();
        self.ask_levels = book.asks.len();
        self.last_update_timestamp = Some(book.timestamp.clone());

        self.best_bid = best_bid_from_book(book);
        self.best_ask = best_ask_from_book(book);
        self.spread = calculate_spread(self.best_bid.as_deref(), self.best_ask.as_deref());
    }

    pub fn apply_best_bid_ask(&mut self, best_bid_ask: &BestBidAsk) {
        if best_bid_ask.asset_id != self.token_id {
            return;
        }

        self.market_id = Some(best_bid_ask.market.clone());
        self.best_bid = Some(best_bid_ask.best_bid.clone());
        self.best_ask = Some(best_bid_ask.best_ask.clone());
        self.spread = Some(best_bid_ask.spread.clone());
        self.last_update_timestamp = Some(best_bid_ask.timestamp.clone());
    }

    pub fn apply_last_trade_price(&mut self, trade: &LastTradePrice) {
        if trade.asset_id != self.token_id {
            return;
        }

        self.market_id = Some(trade.market.clone());
        self.last_trade_price = Some(trade.price.clone());
        self.last_trade_size = Some(trade.size.clone());
        self.last_trade_side = Some(trade.side.clone());
        self.last_update_timestamp = Some(trade.timestamp.clone());
    }

    pub fn apply_price_change(&mut self, price_change: &PriceChange) {
        self.market_id = Some(price_change.market.clone());
        self.last_update_timestamp = Some(price_change.timestamp.clone());

        for change in &price_change.changes {
            if change.asset_id != self.token_id {
                continue;
            }

            if let Some(best_bid) = &change.best_bid {
                self.best_bid = Some(best_bid.clone());
            }

            if let Some(best_ask) = &change.best_ask {
                self.best_ask = Some(best_ask.clone());
            }

            self.spread = calculate_spread(self.best_bid.as_deref(), self.best_ask.as_deref());
        }
    }

    pub fn display_summary(&self) {
        println!(
            "STATE token={} bid={} ask={} spread={} last_trade={} side={} size={} updated={}",
            shorten_token_id(&self.token_id),
            self.best_bid.as_deref().unwrap_or("-"),
            self.best_ask.as_deref().unwrap_or("-"),
            self.spread.as_deref().unwrap_or("-"),
            self.last_trade_price.as_deref().unwrap_or("-"),
            self.last_trade_side.as_deref().unwrap_or("-"),
            self.last_trade_size.as_deref().unwrap_or("-"),
            self.last_update_timestamp.as_deref().unwrap_or("-"),
        );
    }
}

fn best_bid_from_book(book: &OrderBookSnapshot) -> Option<String> {
    book.bids
        .iter()
        .filter_map(|level| level.price.parse::<f64>().ok().map(|price| (price, level)))
        .max_by(|(left_price, _), (right_price, _)| left_price.total_cmp(right_price))
        .map(|(_, level)| level.price.clone())
}

fn best_ask_from_book(book: &OrderBookSnapshot) -> Option<String> {
    book.asks
        .iter()
        .filter_map(|level| level.price.parse::<f64>().ok().map(|price| (price, level)))
        .min_by(|(left_price, _), (right_price, _)| left_price.total_cmp(right_price))
        .map(|(_, level)| level.price.clone())
}

fn calculate_spread(best_bid: Option<&str>, best_ask: Option<&str>) -> Option<String> {
    let best_bid = best_bid?.parse::<f64>().ok()?;
    let best_ask = best_ask?.parse::<f64>().ok()?;

    Some(format!("{:.4}", best_ask - best_bid))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polymarket::ws_types::{BookLevel, PriceLevelChange};

    #[test]
    fn applies_best_bid_ask_update() {
        let mut state = TokenMarketState::new("token-1");

        let event = BestBidAsk {
            market: "market-1".to_string(),
            asset_id: "token-1".to_string(),
            best_bid: "0.42".to_string(),
            best_ask: "0.44".to_string(),
            spread: "0.02".to_string(),
            timestamp: "123456".to_string(),
        };

        state.apply_best_bid_ask(&event);

        assert_eq!(state.market_id.as_deref(), Some("market-1"));
        assert_eq!(state.best_bid.as_deref(), Some("0.42"));
        assert_eq!(state.best_ask.as_deref(), Some("0.44"));
        assert_eq!(state.spread.as_deref(), Some("0.02"));
        assert_eq!(state.last_update_timestamp.as_deref(), Some("123456"));
    }

    #[test]
    fn ignores_best_bid_ask_for_different_token() {
        let mut state = TokenMarketState::new("token-1");

        let event = BestBidAsk {
            market: "market-1".to_string(),
            asset_id: "token-2".to_string(),
            best_bid: "0.42".to_string(),
            best_ask: "0.44".to_string(),
            spread: "0.02".to_string(),
            timestamp: "123456".to_string(),
        };

        state.apply_best_bid_ask(&event);

        assert!(state.best_bid.is_none());
        assert!(state.best_ask.is_none());
        assert!(state.spread.is_none());
    }

    #[test]
    fn applies_last_trade_price_update() {
        let mut state = TokenMarketState::new("token-1");

        let event = LastTradePrice {
            market: "market-1".to_string(),
            asset_id: "token-1".to_string(),
            price: "0.45".to_string(),
            side: "BUY".to_string(),
            size: "100".to_string(),
            timestamp: "123456".to_string(),
            fee_rate_bps: None,
        };

        state.apply_last_trade_price(&event);

        assert_eq!(state.last_trade_price.as_deref(), Some("0.45"));
        assert_eq!(state.last_trade_side.as_deref(), Some("BUY"));
        assert_eq!(state.last_trade_size.as_deref(), Some("100"));
    }

    #[test]
    fn applies_book_snapshot_update() {
        let mut state = TokenMarketState::new("token-1");

        let event = OrderBookSnapshot {
            market: "market-1".to_string(),
            asset_id: "token-1".to_string(),
            timestamp: "123456".to_string(),
            bids: vec![
                BookLevel {
                    price: "0.40".to_string(),
                    size: "10".to_string(),
                },
                BookLevel {
                    price: "0.42".to_string(),
                    size: "20".to_string(),
                },
            ],
            asks: vec![
                BookLevel {
                    price: "0.45".to_string(),
                    size: "10".to_string(),
                },
                BookLevel {
                    price: "0.44".to_string(),
                    size: "20".to_string(),
                },
            ],
            hash: None,
        };

        state.apply_book_snapshot(&event);

        assert_eq!(state.best_bid.as_deref(), Some("0.42"));
        assert_eq!(state.best_ask.as_deref(), Some("0.44"));
        assert_eq!(state.spread.as_deref(), Some("0.0200"));
        assert_eq!(state.bid_levels, 2);
        assert_eq!(state.ask_levels, 2);
    }

    #[test]
    fn applies_price_change_best_bid_and_ask() {
        let mut state = TokenMarketState::new("token-1");

        let event = PriceChange {
            market: "market-1".to_string(),
            timestamp: "123456".to_string(),
            changes: vec![PriceLevelChange {
                asset_id: "token-1".to_string(),
                price: "0.43".to_string(),
                size: "50".to_string(),
                side: "BUY".to_string(),
                hash: None,
                best_bid: Some("0.43".to_string()),
                best_ask: Some("0.45".to_string()),
            }],
        };

        state.apply_price_change(&event);

        assert_eq!(state.best_bid.as_deref(), Some("0.43"));
        assert_eq!(state.best_ask.as_deref(), Some("0.45"));
        assert_eq!(state.spread.as_deref(), Some("0.0200"));
    }
}
