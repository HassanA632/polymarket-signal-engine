use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::polymarket::client::PolymarketClient;
use crate::polymarket::config::load_stream_config;
use crate::polymarket::display::{
    display_events, display_market_inspection_by_market_id, display_market_inspection_by_token_id,
};
use crate::polymarket::signals::{SignalConfig, SignalOutputMode};
use crate::polymarket::stream::{PaperTradingConfig, StreamOutputConfig, stream_token};

mod polymarket;

#[derive(Parser)]
#[command(name = "polymarket-signal-engine")]
#[command(about = "A low-latency Polymarket market-data and signal engine built in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch and display active Polymarket markets
    Markets {
        /// Number of active events to fetch
        #[arg(short, long, default_value_t = 10)]
        limit: u32,

        /// Maximum number of tradable markets to display per event
        #[arg(long, default_value_t = 10)]
        max_display_markets: usize,

        /// Search events and markets by keyword
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Inspect a specific market by market ID
    Inspect {
        /// Polymarket market ID to inspect
        #[arg(long)]
        market_id: Option<String>,

        /// CLOB token ID to inspect
        #[arg(long)]
        token_id: Option<String>,

        /// Number of active events to search through
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },
    /// Stream live market data for a CLOB token ID
    Stream {
        /// CLOB token ID to stream
        #[arg(long)]
        token_id: String,

        /// Optional TOML config file for stream settings
        #[arg(long)]
        config: Option<PathBuf>,

        /// Maximum spread allowed for a TightSpread signal
        #[arg(long)]
        tight_spread_threshold: Option<f64>,

        /// Minimum spread reduction required for a SpreadTightened signal
        #[arg(long)]
        min_spread_tightening: Option<f64>,

        /// Minimum best-bid movement required for a price movement signal
        #[arg(long)]
        min_price_move: Option<f64>,

        /// Minimum trade size required for a LargeTrade signal
        #[arg(long)]
        large_trade_threshold: Option<f64>,

        /// Signal output format
        #[arg(long, value_enum)]
        output: Option<OutputMode>,

        /// Optional path to write emitted signals as JSONL
        #[arg(long)]
        log_signals: Option<PathBuf>,

        /// Show live token state summaries
        #[arg(long)]
        show_state: Option<bool>,

        /// Show parsed WebSocket event summaries
        #[arg(long)]
        show_events: Option<bool>,

        /// Enable experimental paper trading mode
        #[arg(long)]
        paper_trade: Option<bool>,

        /// Stake size used for simulated paper trades
        #[arg(long)]
        paper_stake: Option<f64>,

        /// Paper-trading take-profit threshold as a decimal, e.g. 0.05 = 5%
        #[arg(long)]
        take_profit: Option<f64>,

        /// Paper-trading stop-loss threshold as a decimal, e.g. 0.03 = 3%
        #[arg(long)]
        stop_loss: Option<f64>,

        /// Optional path to write closed paper trades as JSONL
        #[arg(long)]
        log_paper_trades: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputMode {
    Text,
    Json,
}

impl From<OutputMode> for SignalOutputMode {
    fn from(value: OutputMode) -> Self {
        match value {
            OutputMode::Text => SignalOutputMode::Text,
            OutputMode::Json => SignalOutputMode::Json,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Markets {
            limit,
            max_display_markets,
            search,
        } => {
            tracing::info!(limit, "Fetching active Polymarket events");

            let client = PolymarketClient::new();
            let events = client.fetch_active_events(limit).await?;

            display_events(&events, max_display_markets, search.as_deref());
        }

        Commands::Inspect {
            market_id,
            token_id,
            limit,
        } => {
            tracing::info!(
                market_id = ?market_id,
                token_id = ?token_id,
                limit,
                "Inspecting Polymarket market"
            );

            let client = PolymarketClient::new();
            let events = client.fetch_active_events(limit).await?;

            match (market_id.as_deref(), token_id.as_deref()) {
                (Some(market_id), None) => {
                    display_market_inspection_by_market_id(&events, market_id);
                }
                (None, Some(token_id)) => {
                    display_market_inspection_by_token_id(&events, token_id);
                }
                (None, None) => {
                    eprintln!("Please provide either --market-id or --token-id.");
                }
                (Some(_), Some(_)) => {
                    eprintln!("Please provide only one of --market-id or --token-id, not both.");
                }
            }
        }
        Commands::Stream {
            token_id,
            config,
            tight_spread_threshold,
            min_spread_tightening,
            min_price_move,
            large_trade_threshold,
            output,
            log_signals,
            show_state,
            show_events,
            paper_trade,
            paper_stake,
            take_profit,
            stop_loss,
            log_paper_trades,
        } => {
            let file_config = match config {
                Some(path) => Some(load_stream_config(path)?),
                None => None,
            };

            let signal_file_config = file_config
                .as_ref()
                .and_then(|config| config.signals.as_ref());
            let output_file_config = file_config
                .as_ref()
                .and_then(|config| config.output.as_ref());
            let paper_file_config = file_config
                .as_ref()
                .and_then(|config| config.paper_trading.as_ref());

            let signal_config = SignalConfig {
                tight_spread_threshold: tight_spread_threshold
                    .or_else(|| signal_file_config.and_then(|config| config.tight_spread_threshold))
                    .unwrap_or_else(|| SignalConfig::default().tight_spread_threshold),

                min_spread_tightening: min_spread_tightening
                    .or_else(|| signal_file_config.and_then(|config| config.min_spread_tightening))
                    .unwrap_or_else(|| SignalConfig::default().min_spread_tightening),

                min_price_move: min_price_move
                    .or_else(|| signal_file_config.and_then(|config| config.min_price_move))
                    .unwrap_or_else(|| SignalConfig::default().min_price_move),

                large_trade_threshold: large_trade_threshold
                    .or_else(|| signal_file_config.and_then(|config| config.large_trade_threshold))
                    .unwrap_or_else(|| SignalConfig::default().large_trade_threshold),
            };

            fn parse_output_mode(value: &str) -> Option<SignalOutputMode> {
                match value.to_lowercase().as_str() {
                    "text" => Some(SignalOutputMode::Text),
                    "json" => Some(SignalOutputMode::Json),
                    _ => None,
                }
            }

            let output_mode = output
                .map(SignalOutputMode::from)
                .or_else(|| {
                    output_file_config
                        .and_then(|config| config.mode.as_deref())
                        .and_then(parse_output_mode)
                })
                .unwrap_or(SignalOutputMode::Text);

            let output_config = StreamOutputConfig {
                show_state: show_state
                    .or_else(|| output_file_config.and_then(|config| config.show_state))
                    .unwrap_or(false),

                show_events: show_events
                    .or_else(|| output_file_config.and_then(|config| config.show_events))
                    .unwrap_or(false),
            };

            let signal_log_path = log_signals.or_else(|| {
                output_file_config
                    .and_then(|config| config.log_signals.as_ref())
                    .cloned()
            });

            let paper_trading_config = PaperTradingConfig {
                enabled: paper_trade
                    .or_else(|| paper_file_config.and_then(|config| config.enabled))
                    .unwrap_or(false),

                stake: paper_stake
                    .or_else(|| paper_file_config.and_then(|config| config.stake))
                    .unwrap_or(10.0),

                take_profit: take_profit
                    .or_else(|| paper_file_config.and_then(|config| config.take_profit))
                    .unwrap_or(0.05),

                stop_loss: stop_loss
                    .or_else(|| paper_file_config.and_then(|config| config.stop_loss))
                    .unwrap_or(0.03),
            };

            let paper_trade_log_path = log_paper_trades.or_else(|| {
                paper_file_config
                    .and_then(|config| config.log_paper_trades.as_ref())
                    .cloned()
            });

            stream_token(
                &token_id,
                signal_config,
                output_mode,
                signal_log_path,
                output_config,
                paper_trading_config,
                paper_trade_log_path,
            )
            .await?;
        }
    }

    Ok(())
}
