use crate::polymarket::state::TokenMarketState;

#[derive(Debug, Clone, PartialEq)]
pub enum MarketSignal {
    TightSpread {
        token_id: String,
        best_bid: String,
        best_ask: String,
        spread: String,
    },
}

pub fn evaluate_signals(state: &TokenMarketState) -> Vec<MarketSignal> {
    let mut signals = Vec::new();

    if let Some(signal) = evaluate_tight_spread(state, 0.01) {
        signals.push(signal);
    }

    signals
}

fn evaluate_tight_spread(state: &TokenMarketState, max_spread: f64) -> Option<MarketSignal> {
    let best_bid = state.best_bid.as_ref()?;
    let best_ask = state.best_ask.as_ref()?;
    let spread = state.spread.as_ref()?;

    let spread_value = spread.parse::<f64>().ok()?;

    if spread_value <= max_spread {
        return Some(MarketSignal::TightSpread {
            token_id: state.token_id.clone(),
            best_bid: best_bid.clone(),
            best_ask: best_ask.clone(),
            spread: spread.clone(),
        });
    }

    None
}

pub fn display_signal(signal: &MarketSignal) {
    match signal {
        MarketSignal::TightSpread {
            token_id,
            best_bid,
            best_ask,
            spread,
        } => {
            println!(
                "SIGNAL TightSpread token={} bid={} ask={} spread={}",
                shorten_token_id(token_id),
                best_bid,
                best_ask,
                spread
            );
        }
    }
}

fn shorten_token_id(token_id: &str) -> String {
    const PREFIX_LEN: usize = 8;
    const SUFFIX_LEN: usize = 6;

    if token_id.len() <= PREFIX_LEN + SUFFIX_LEN {
        return token_id.to_string();
    }

    format!(
        "{}...{}",
        &token_id[..PREFIX_LEN],
        &token_id[token_id.len() - SUFFIX_LEN..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_spread(spread: &str) -> TokenMarketState {
        let mut state = TokenMarketState::new("token-1");

        state.best_bid = Some("0.42".to_string());
        state.best_ask = Some("0.43".to_string());
        state.spread = Some(spread.to_string());

        state
    }

    #[test]
    fn emits_tight_spread_signal_when_spread_is_below_threshold() {
        let state = state_with_spread("0.005");

        let signals = evaluate_signals(&state);

        assert_eq!(signals.len(), 1);

        assert_eq!(
            signals[0],
            MarketSignal::TightSpread {
                token_id: "token-1".to_string(),
                best_bid: "0.42".to_string(),
                best_ask: "0.43".to_string(),
                spread: "0.005".to_string(),
            }
        );
    }

    #[test]
    fn emits_tight_spread_signal_when_spread_equals_threshold() {
        let state = state_with_spread("0.01");

        let signals = evaluate_signals(&state);

        assert_eq!(signals.len(), 1);
    }

    #[test]
    fn does_not_emit_tight_spread_signal_when_spread_is_above_threshold() {
        let state = state_with_spread("0.02");

        let signals = evaluate_signals(&state);

        assert!(signals.is_empty());
    }

    #[test]
    fn does_not_emit_signal_when_bid_or_ask_is_missing() {
        let mut state = state_with_spread("0.005");
        state.best_bid = None;

        let signals = evaluate_signals(&state);

        assert!(signals.is_empty());
    }

    #[test]
    fn does_not_emit_signal_when_spread_is_invalid() {
        let state = state_with_spread("not-a-number");

        let signals = evaluate_signals(&state);

        assert!(signals.is_empty());
    }
}
