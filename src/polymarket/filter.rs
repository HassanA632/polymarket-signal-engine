use crate::polymarket::types::Event;

pub fn event_matches_search(event: &Event, search_term: &str) -> bool {
    let search_term = search_term.to_lowercase();

    event.title.to_lowercase().contains(&search_term)
        || event.slug.to_lowercase().contains(&search_term)
        || event.markets.iter().any(|market| {
            market
                .question
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(&search_term)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polymarket::types::{Event, Market};

    fn test_event() -> Event {
        Event {
            id: "event-1".to_string(),
            title: "Bitcoin price predictions".to_string(),
            slug: "bitcoin-price-predictions".to_string(),
            volume: Some(1000.0),
            liquidity: Some(500.0),
            markets: vec![
                Market {
                    id: "market-1".to_string(),
                    question: Some("Will Bitcoin hit $100k in 2026?".to_string()),
                    condition_id: Some("condition-1".to_string()),
                    clob_token_ids: Some("[\"yes-token\", \"no-token\"]".to_string()),
                    outcomes: Some("[\"Yes\", \"No\"]".to_string()),
                    active: Some(true),
                    closed: Some(false),
                },
                Market {
                    id: "market-2".to_string(),
                    question: Some("Will Ethereum hit $10k in 2026?".to_string()),
                    condition_id: Some("condition-2".to_string()),
                    clob_token_ids: Some("[\"yes-token-2\", \"no-token-2\"]".to_string()),
                    outcomes: Some("[\"Yes\", \"No\"]".to_string()),
                    active: Some(true),
                    closed: Some(false),
                },
            ],
        }
    }

    #[test]
    fn matches_event_title() {
        let event = test_event();

        assert!(event_matches_search(&event, "bitcoin"));
    }

    #[test]
    fn matches_event_slug() {
        let event = test_event();

        assert!(event_matches_search(&event, "price-predictions"));
    }

    #[test]
    fn matches_child_market_question() {
        let event = test_event();

        assert!(event_matches_search(&event, "ethereum"));
    }

    #[test]
    fn search_is_case_insensitive() {
        let event = test_event();

        assert!(event_matches_search(&event, "BITCOIN"));
    }

    #[test]
    fn returns_false_when_no_match() {
        let event = test_event();

        assert!(!event_matches_search(&event, "tennis"));
    }

    #[test]
    fn handles_missing_market_question() {
        let mut event = test_event();
        event.markets[0].question = None;

        assert!(event_matches_search(&event, "ethereum"));
        assert!(!event_matches_search(&event, "missing-question-text"));
    }
}
