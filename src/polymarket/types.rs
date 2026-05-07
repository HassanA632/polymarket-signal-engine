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
    pub active: Option<bool>,

    #[serde(default)]
    pub closed: Option<bool>,
}

impl Market {
    pub fn is_tradable(&self) -> bool {
        self.active == Some(true) && self.closed == Some(false)
    }

    pub fn parsed_clob_token_ids(&self) -> Vec<String> {
        match &self.clob_token_ids {
            Some(raw_token_ids) => {
                serde_json::from_str::<Vec<String>>(raw_token_ids).unwrap_or_default()
            }
            None => Vec::new(),
        }
    }
}
