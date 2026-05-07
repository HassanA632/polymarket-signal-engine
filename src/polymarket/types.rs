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
