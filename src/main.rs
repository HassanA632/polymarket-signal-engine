use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::polymarket::client::PolymarketClient;
use crate::polymarket::display::display_events;

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
    }

    Ok(())
}
