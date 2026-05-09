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
