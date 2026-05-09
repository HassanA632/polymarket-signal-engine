use crate::polymarket::types::{Event, Market};

pub fn display_events(events: &[Event], max_display_markets: usize, search: Option<&str>) {
    let search = search.map(|value| value.to_lowercase());
    let mut displayed_events = 0;

    for event in events {
        if let Some(search_term) = &search {
            if !event_matches_search(event, search_term) {
                continue;
            }
        }

        displayed_events += 1;

        println!("{}. {}", displayed_events, event.title);
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

    if displayed_events == 0 {
        println!("No matching events found.");
    }
}

fn event_matches_search(event: &Event, search_term: &str) -> bool {
    event.title.to_lowercase().contains(search_term)
        || event.slug.to_lowercase().contains(search_term)
        || event.markets.iter().any(|market| {
            market
                .question
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(search_term)
        })
}

fn display_market(market: &Market) {
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
