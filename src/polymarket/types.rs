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

    #[serde(default)]
    pub active: Option<bool>,

    #[serde(default)]
    pub closed: Option<bool>,
}
