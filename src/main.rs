use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Polymarket Signal Engine");
    println!("Starting CLI...");
    Ok(())
}
