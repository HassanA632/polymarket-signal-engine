use crate::polymarket::types::Event;

pub fn display_events(events: &[Event], max_display_markets: usize) {
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

        for market in tradable_markets.iter().take(max_display_markets) {
            display_market(market);
        }

        let hidden_count = tradable_markets.len().saturating_sub(max_display_markets);

        if hidden_count > 0 {
            println!(
                "   ... {} more tradable markets not displayed",
                hidden_count
            );
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

    let outcome_tokens = market.outcome_tokens();

    if !outcome_tokens.is_empty() {
        println!("     outcome tokens:");

        for outcome_token in outcome_tokens {
            println!(
                "       {} -> {}",
                outcome_token.outcome, outcome_token.token_id
            );
        }
    }

    if let Some(active) = market.active {
        println!("     active: {}", active);
    }

    if let Some(closed) = market.closed {
        println!("     closed: {}", closed);
    }
}
