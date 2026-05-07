use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::polymarket::client::PolymarketClient;

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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Markets { limit } => {
            tracing::info!(limit, "Fetching active Polymarket events");

            let client = PolymarketClient::new();
            let events = client.fetch_active_events(limit).await?;

            for (index, event) in events.iter().enumerate() {
                println!("{}. {}", index + 1, event.title);
                println!("   slug: {}", event.slug);
                println!("   markets: {}", event.markets.len());

                if let Some(volume) = event.volume {
                    println!("   volume: {:.2}", volume);
                }

                if let Some(liquidity) = event.liquidity {
                    println!("   liquidity: {:.2}", liquidity);
                }

                for market in &event.markets {
                    println!(
                        "   - {}",
                        market.question.as_deref().unwrap_or("Unknown question")
                    );
                    println!("     market id: {}", market.id);

                    if let Some(condition_id) = &market.condition_id {
                        println!("     condition id: {}", condition_id);
                    }

                    if let Some(active) = market.active {
                        println!("     active: {}", active);
                    }

                    if let Some(closed) = market.closed {
                        println!("     closed: {}", closed);
                    }
                }

                println!();
            }
        }
    }

    Ok(())
}
