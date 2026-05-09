use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MarketMessageEnvelope {
    pub event_type: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderBookSnapshot {
    pub market: String,
    pub asset_id: String,
    pub timestamp: String,

    #[serde(default)]
    pub bids: Vec<BookLevel>,

    #[serde(default)]
    pub asks: Vec<BookLevel>,

    #[serde(default)]
    pub hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookLevel {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Deserialize)]
pub struct PriceChange {
    pub market: String,
    pub timestamp: String,

    #[serde(rename = "price_changes")]
    #[serde(default)]
    pub changes: Vec<PriceLevelChange>,
}

#[derive(Debug, Deserialize)]
pub struct PriceLevelChange {
    pub asset_id: String,
    pub price: String,
    pub size: String,
    pub side: String,

    #[serde(default)]
    pub hash: Option<String>,

    #[serde(default)]
    pub best_bid: Option<String>,

    #[serde(default)]
    pub best_ask: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LastTradePrice {
    pub market: String,
    pub asset_id: String,
    pub price: String,
    pub side: String,
    pub size: String,
    pub timestamp: String,

    #[serde(default)]
    pub fee_rate_bps: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BestBidAsk {
    pub market: String,
    pub asset_id: String,
    pub best_bid: String,
    pub best_ask: String,
    pub spread: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct TickSizeChange {
    pub market: String,
    pub asset_id: String,
    pub old_tick_size: String,
    pub new_tick_size: String,
    pub timestamp: String,
}
