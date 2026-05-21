use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct StreamConfigFile {
    pub signals: Option<SignalConfigFile>,
    pub output: Option<OutputConfigFile>,
    pub paper_trading: Option<PaperTradingConfigFile>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SignalConfigFile {
    pub tight_spread_threshold: Option<f64>,
    pub min_spread_tightening: Option<f64>,
    pub min_price_move: Option<f64>,
    pub large_trade_threshold: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OutputConfigFile {
    pub mode: Option<String>,
    pub show_state: Option<bool>,
    pub show_events: Option<bool>,
    pub log_signals: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PaperTradingConfigFile {
    pub enabled: Option<bool>,
    pub stake: Option<f64>,
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
    pub log_paper_trades: Option<PathBuf>,
}

pub fn load_stream_config(path: impl AsRef<Path>) -> Result<StreamConfigFile> {
    let contents = fs::read_to_string(path)?;
    let config = toml::from_str::<StreamConfigFile>(&contents)?;

    Ok(config)
}
