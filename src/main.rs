use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::polymarket::client::PolymarketClient;
use crate::polymarket::display::{
    display_events, display_market_inspection_by_market_id, display_market_inspection_by_token_id,
};
use crate::polymarket::signals::SignalConfig;
use crate::polymarket::stream::stream_token;

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

        /// Maximum spread allowed for a TightSpread signal
        #[arg(long, default_value_t = 0.01)]
        tight_spread_threshold: f64,

        /// Minimum spread reduction required for a SpreadTightened signal
        #[arg(long, default_value_t = 0.01)]
        min_spread_tightening: f64,

        /// Minimum best-bid movement required for a price movement signal
        #[arg(long, default_value_t = 0.02)]
        min_price_move: f64,
    },
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
            tight_spread_threshold,
            min_spread_tightening,
            min_price_move,
        } => {
            let signal_config = SignalConfig {
                tight_spread_threshold,
                min_spread_tightening,
                min_price_move,
            };

            stream_token(&token_id, signal_config).await?;
        }
    }

    Ok(())
}
