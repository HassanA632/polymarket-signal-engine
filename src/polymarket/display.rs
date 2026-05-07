use crate::polymarket::types::Event;

pub fn display_events(events: &[Event]) {
    for (index, event) in events.iter().enumerate() {
        println!("{}. {}", index + 1, event.title);
        println!("   slug: {}", event.slug);

        let tradable_markets: Vec<_> = event
            .markets
            .iter()
            .filter(|market| market.is_tradable())
            .collect();

        println!(
            "   tradable markets: {}/{}",
            tradable_markets.len(),
            event.markets.len()
        );

        if let Some(volume) = event.volume {
            println!("   volume: {:.2}", volume);
        }

        if let Some(liquidity) = event.liquidity {
            println!("   liquidity: {:.2}", liquidity);
        }

        for market in tradable_markets {
            display_market(market);
        }

        println!();
    }
}

fn display_market(market: &crate::polymarket::types::Market) {
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
