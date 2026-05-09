use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub slug: String,

    #[serde(default)]
    pub volume: Option<f64>,

    #[serde(default)]
    pub liquidity: Option<f64>,

    #[serde(default)]
    pub markets: Vec<Market>,
}

#[derive(Debug, Deserialize)]
pub struct Market {
    pub id: String,

    #[serde(default)]
    pub question: Option<String>,

    #[serde(rename = "conditionId")]
    #[serde(default)]
    pub condition_id: Option<String>,

    #[serde(rename = "clobTokenIds")]
    #[serde(default)]
    pub clob_token_ids: Option<String>,

    #[serde(default)]
    pub outcomes: Option<String>,

    #[serde(default)]
    pub active: Option<bool>,

    #[serde(default)]
    pub closed: Option<bool>,
}

#[derive(Debug)]
pub struct OutcomeToken {
    pub outcome: String,
    pub token_id: String,
}
impl Market {
    pub fn is_tradable(&self) -> bool {
        self.active == Some(true) && self.closed == Some(false)
    }

    pub fn parsed_clob_token_ids(&self) -> Vec<String> {
        parse_json_string_array(self.clob_token_ids.as_deref())
    }

    pub fn parsed_outcomes(&self) -> Vec<String> {
        parse_json_string_array(self.outcomes.as_deref())
    }

    pub fn outcome_tokens(&self) -> Vec<OutcomeToken> {
        let outcomes = self.parsed_outcomes();
        let token_ids = self.parsed_clob_token_ids();

        outcomes
            .into_iter()
            .zip(token_ids)
            .map(|(outcome, token_id)| OutcomeToken { outcome, token_id })
            .collect()
    }
}

fn parse_json_string_array(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_market() -> Market {
        Market {
            id: "123".to_string(),
            question: Some("Will Bitcoin hit $100k?".to_string()),
            condition_id: Some("condition-1".to_string()),
            clob_token_ids: Some("[\"yes-token\", \"no-token\"]".to_string()),
            outcomes: Some("[\"Yes\", \"No\"]".to_string()),
            active: Some(true),
            closed: Some(false),
        }
    }

    #[test]
    fn market_is_tradable_when_active_and_not_closed() {
        let market = test_market();

        assert!(market.is_tradable());
    }

    #[test]
    fn market_is_not_tradable_when_inactive() {
        let mut market = test_market();
        market.active = Some(false);

        assert!(!market.is_tradable());
    }

    #[test]
    fn market_is_not_tradable_when_closed() {
        let mut market = test_market();
        market.closed = Some(true);

        assert!(!market.is_tradable());
    }

    #[test]
    fn parses_clob_token_ids() {
        let market = test_market();

        assert_eq!(
            market.parsed_clob_token_ids(),
            vec!["yes-token".to_string(), "no-token".to_string()]
        );
    }

    #[test]
    fn parses_outcomes() {
        let market = test_market();

        assert_eq!(
            market.parsed_outcomes(),
            vec!["Yes".to_string(), "No".to_string()]
        );
    }

    #[test]
    fn pairs_outcomes_with_token_ids() {
        let market = test_market();
        let outcome_tokens = market.outcome_tokens();

        assert_eq!(outcome_tokens.len(), 2);

        assert_eq!(outcome_tokens[0].outcome, "Yes");
        assert_eq!(outcome_tokens[0].token_id, "yes-token");

        assert_eq!(outcome_tokens[1].outcome, "No");
        assert_eq!(outcome_tokens[1].token_id, "no-token");
    }

    #[test]
    fn invalid_token_json_returns_empty_vec() {
        let mut market = test_market();
        market.clob_token_ids = Some("not-valid-json".to_string());

        assert!(market.parsed_clob_token_ids().is_empty());
    }

    #[test]
    fn missing_outcomes_returns_empty_vec() {
        let mut market = test_market();
        market.outcomes = None;

        assert!(market.parsed_outcomes().is_empty());
    }
}
