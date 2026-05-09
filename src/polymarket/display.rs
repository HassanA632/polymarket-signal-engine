use crate::polymarket::filter::event_matches_search;
use crate::polymarket::types::{Event, Market};

pub fn display_events(events: &[Event], max_display_markets: usize, search: Option<&str>) {
    let mut displayed_events = 0;

    for event in events {
        if let Some(search_term) = search {
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

pub fn display_market_inspection_by_market_id(events: &[Event], market_id: &str) {
    for event in events {
        if let Some(market) = event.markets.iter().find(|market| market.id == market_id) {
            println!("Market Inspection");
            println!("=================");
            println!("event: {}", event.title);
            println!("event slug: {}", event.slug);

            if let Some(volume) = event.volume {
                println!("event volume: {:.2}", volume);
            }

            if let Some(liquidity) = event.liquidity {
                println!("event liquidity: {:.2}", liquidity);
            }

            println!();
            display_market(market);

            return;
        }
    }

    println!("No market found with id: {}", market_id);
    println!("Try increasing the search limit, for example:");
    println!("cargo run -- inspect --market-id {} --limit 500", market_id);
}

pub fn display_market_inspection_by_token_id(events: &[Event], token_id: &str) {
    for event in events {
        for market in &event.markets {
            let matching_outcome_token = market
                .outcome_tokens()
                .into_iter()
                .find(|outcome_token| outcome_token.token_id == token_id);

            if let Some(outcome_token) = matching_outcome_token {
                println!("Token Inspection");
                println!("================");
                println!("matched outcome: {}", outcome_token.outcome);
                println!("matched token id: {}", outcome_token.token_id);
                println!();

                println!("event: {}", event.title);
                println!("event slug: {}", event.slug);

                if let Some(volume) = event.volume {
                    println!("event volume: {:.2}", volume);
                }

                if let Some(liquidity) = event.liquidity {
                    println!("event liquidity: {:.2}", liquidity);
                }

                println!();
                display_market(market);

                return;
            }
        }
    }

    println!("No market found with CLOB token id: {}", token_id);
    println!("Try increasing the search limit, for example:");
    println!("cargo run -- inspect --token-id {} --limit 500", token_id);
}
